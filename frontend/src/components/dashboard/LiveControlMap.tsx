import { MapContainer, TileLayer, Marker, Popup, useMap, useMapEvents } from 'react-leaflet';
import L from 'leaflet';
import 'leaflet/dist/leaflet.css';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogTrigger } from '@/components/ui/dialog';
import { AgentLocationDto } from '@/openapi-rq/requests/types.gen';
import { formatDistanceToNow } from 'date-fns';
import { fr as frLocale } from 'date-fns/locale';
import { Expand, Map as MapIcon } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';

// Fix for default marker icons in Leaflet with Webpack/Vite
import markerIcon2x from 'leaflet/dist/images/marker-icon-2x.png';
import markerIcon from 'leaflet/dist/images/marker-icon.png';
import markerShadow from 'leaflet/dist/images/marker-shadow.png';

const DEFAULT_CENTER: [number, number] = [3.848, 11.5021]; // Yaounde
const DEFAULT_ZOOM = 12;

const leafletIconProto = L.Icon.Default.prototype as unknown as Record<string, unknown>;
delete leafletIconProto._getIconUrl;
L.Icon.Default.mergeOptions({
  iconUrl: markerIcon,
  iconRetinaUrl: markerIcon2x,
  shadowUrl: markerShadow,
});

// Custom icon for agents
const agentIcon = new L.Icon({
  iconUrl: markerIcon,
  iconRetinaUrl: markerIcon2x,
  shadowUrl: markerShadow,
  iconSize: [25, 41],
  iconAnchor: [12, 41],
  popupAnchor: [1, -34],
  tooltipAnchor: [16, -28],
  shadowSize: [41, 41],
  className: 'agent-marker-icon',
});

interface LiveControlMapProps {
  agents: AgentLocationDto[];
  center?: [number, number];
  zoom?: number;
}

function MapAutoFit({ agents }: { agents: AgentLocationDto[] }) {
  const map = useMap();
  const hasFitRef = useRef(false);

  const bounds = useMemo(() => {
    const points = agents
      .filter((a) => Number.isFinite(a.latitude) && Number.isFinite(a.longitude))
      .map((a) => L.latLng(a.latitude, a.longitude));
    if (points.length === 0) return null;
    return L.latLngBounds(points);
  }, [agents]);

  useEffect(() => {
    if (hasFitRef.current) return;
    if (!bounds) return;

    hasFitRef.current = true;
    map.fitBounds(bounds, {
      padding: [24, 24],
      maxZoom: 14,
      animate: true,
    });
  }, [bounds, map]);

  return null;
}

function MapViewSync({
  onChange,
}: {
  onChange: (next: { center: [number, number]; zoom: number }) => void;
}) {
  useMapEvents({
    moveend: (e) => {
      const map = e.target;
      const c = map.getCenter();
      onChange({ center: [c.lat, c.lng], zoom: map.getZoom() });
    },
    zoomend: (e) => {
      const map = e.target;
      const c = map.getCenter();
      onChange({ center: [c.lat, c.lng], zoom: map.getZoom() });
    },
  });

  return null;
}

function MapInvalidateSize({ active }: { active: boolean }) {
  const map = useMap();

  useEffect(() => {
    if (!active) return;

    // Leaflet needs this when the map is rendered inside a dialog.
    // Two rAFs is a common trick to ensure layout has settled.
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        map.invalidateSize();
      });
    });
  }, [active, map]);

  return null;
}

export function LiveControlMap({
  agents,
  center = DEFAULT_CENTER,
  zoom = DEFAULT_ZOOM,
}: LiveControlMapProps) {
  const { t, i18n } = useTranslation();
  const dateLocale = i18n.language === 'fr' ? frLocale : undefined;
  const [fullscreenOpen, setFullscreenOpen] = useState(false);
  const [view, setView] = useState<{ center: [number, number]; zoom: number }>({
    center,
    zoom,
  });

  useEffect(() => {
    setView((prev) => {
      const nextCenter: [number, number] = prev.center ?? center;
      const nextZoom = prev.zoom ?? zoom;

      const centerUnchanged =
        prev.center?.[0] === nextCenter[0] && prev.center?.[1] === nextCenter[1];
      const zoomUnchanged = prev.zoom === nextZoom;
      if (centerUnchanged && zoomUnchanged) return prev;

      return {
        center: nextCenter,
        zoom: nextZoom,
      };
    });
  }, [center, zoom]);

  const mostRecentUpdate = useMemo(() => {
    const timestamps = agents
      .map((a) => {
        const ts = new Date(a.lastUpdated).getTime();
        return Number.isFinite(ts) ? ts : null;
      })
      .filter((v): v is number => typeof v === 'number');
    if (timestamps.length === 0) return null;
    return new Date(Math.max(...timestamps));
  }, [agents]);

  return (
    <Card className="col-span-1 lg:col-span-2 overflow-hidden border-none shadow-sm bg-card/50 backdrop-blur-sm">
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between gap-3">
          <CardTitle className="text-lg font-semibold flex items-center gap-2">
            <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
            Live Control Map
          </CardTitle>

          <Dialog open={fullscreenOpen} onOpenChange={setFullscreenOpen}>
            <DialogTrigger asChild>
              <Button variant="ghost" size="sm" className="h-8 gap-2 rounded-xl">
                <Expand className="h-4 w-4" />
                {t('common.fullscreen')}
              </Button>
            </DialogTrigger>
            <DialogContent className="max-w-[95vw] w-[95vw] h-[90vh] p-0 overflow-hidden">
              <div className="flex h-full w-full flex-col">
                <div className="flex items-center justify-between border-b px-4 py-3">
                  <div className="flex items-center gap-2 text-sm font-semibold">
                    <MapIcon className="h-4 w-4" />
                    {t('backOfficeDashboard.liveControlMap')}
                  </div>
                  <div className="text-xs text-muted-foreground">
                    {agents.length} {t('common.agentsOnline').toLowerCase()}
                  </div>
                </div>

                <div className="relative flex-1">
                  <MapContainer
                    center={view.center}
                    zoom={view.zoom}
                    scrollWheelZoom
                    className="h-full w-full z-10"
                  >
                    <MapInvalidateSize active={fullscreenOpen} />
                    <MapViewSync onChange={setView} />
                    <TileLayer
                      attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
                      url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
                    />
                    <MapAutoFit agents={agents} />
                    {agents.map((agent) => (
                      <Marker
                        key={agent.agentId}
                        position={[agent.latitude, agent.longitude]}
                        icon={agentIcon}
                      >
                        <Popup>
                          <div className="p-1">
                            <p className="font-bold text-sm mb-1">{agent.agentName}</p>
                            <p className="text-xs text-muted-foreground">
                              {t('common.lastActive')}:{' '}
                              {formatDistanceToNow(new Date(agent.lastUpdated), {
                                addSuffix: true,
                                locale: dateLocale,
                              })}
                            </p>
                          </div>
                        </Popup>
                      </Marker>
                    ))}
                  </MapContainer>
                </div>
              </div>
            </DialogContent>
          </Dialog>
        </div>
      </CardHeader>
      <CardContent className="p-0 relative h-[400px]">
        <MapContainer
          center={view.center}
          zoom={view.zoom}
          scrollWheelZoom={false}
          className="h-full w-full z-10"
        >
          <MapViewSync onChange={setView} />
          <TileLayer
            attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
            url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
          />
          <MapAutoFit agents={agents} />
          {agents.map((agent) => (
            <Marker
              key={agent.agentId}
              position={[agent.latitude, agent.longitude]}
              icon={agentIcon}
            >
              <Popup>
                <div className="p-1">
                  <p className="font-bold text-sm mb-1">{agent.agentName}</p>
                  <p className="text-xs text-muted-foreground">
                    {t('common.lastActive')}:{' '}
                    {formatDistanceToNow(new Date(agent.lastUpdated), {
                      addSuffix: true,
                      locale: dateLocale,
                    })}
                  </p>
                  <div className="mt-2 flex items-center gap-1">
                    <div className="w-2 h-2 rounded-full bg-green-500" />
                    <span className="text-[10px] uppercase font-bold text-green-600">
                      {t('common.activeNow')}
                    </span>
                  </div>
                </div>
              </Popup>
            </Marker>
          ))}
        </MapContainer>

        {/* Overlay for map details or legend if needed */}
        <div className="absolute bottom-4 left-4 z-20 bg-background/80 backdrop-blur-md p-2 rounded-lg shadow-lg border text-xs">
          <div className="flex flex-col gap-1">
            <div className="flex items-center gap-2">
              <span className="w-3 h-3 rounded-full bg-blue-500" />
              <span>
                {agents.length} {t('common.agentsOnline')}
              </span>
            </div>
            {mostRecentUpdate && (
              <span className="text-[10px] text-muted-foreground">
                {t('common.updated')}{' '}
                {formatDistanceToNow(mostRecentUpdate, { addSuffix: true, locale: dateLocale })}
              </span>
            )}
          </div>
        </div>

        {agents.length === 0 && (
          <div className="absolute inset-0 z-20 flex items-center justify-center">
            <div className="rounded-xl border bg-background/80 backdrop-blur-md px-4 py-3 text-sm text-muted-foreground shadow-sm">
              {t('common.noAgentsOnline')}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
