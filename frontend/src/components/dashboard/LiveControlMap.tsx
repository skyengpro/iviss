import { MapContainer, TileLayer, Marker, Popup } from 'react-leaflet';
import L from 'leaflet';
import 'leaflet/dist/leaflet.css';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { AgentLocationDto } from '@/openapi-rq/requests/types.gen';
import { formatDistanceToNow } from 'date-fns';

// Fix for default marker icons in Leaflet with Webpack/Vite
import markerIcon2x from 'leaflet/dist/images/marker-icon-2x.png';
import markerIcon from 'leaflet/dist/images/marker-icon.png';
import markerShadow from 'leaflet/dist/images/marker-shadow.png';

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

export function LiveControlMap({
  agents,
  center = [3.848, 11.5021], // Default to Yaounde
  zoom = 12,
}: LiveControlMapProps) {
  return (
    <Card className="col-span-1 lg:col-span-2 overflow-hidden border-none shadow-sm bg-card/50 backdrop-blur-sm">
      <CardHeader className="pb-2">
        <CardTitle className="text-lg font-semibold flex items-center gap-2">
          <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
          Live Control Map
        </CardTitle>
      </CardHeader>
      <CardContent className="p-0 relative h-[400px]">
        <MapContainer
          center={center}
          zoom={zoom}
          scrollWheelZoom={false}
          className="h-full w-full z-10"
        >
          <TileLayer
            attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
            url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
          />
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
                    Last active:{' '}
                    {formatDistanceToNow(new Date(agent.lastUpdated), { addSuffix: true })}
                  </p>
                  <div className="mt-2 flex items-center gap-1">
                    <div className="w-2 h-2 rounded-full bg-green-500" />
                    <span className="text-[10px] uppercase font-bold text-green-600">
                      Active Now
                    </span>
                  </div>
                </div>
              </Popup>
            </Marker>
          ))}
        </MapContainer>

        {/* Overlay for map details or legend if needed */}
        <div className="absolute bottom-4 left-4 z-20 bg-background/80 backdrop-blur-md p-2 rounded-lg shadow-lg border text-xs">
          <div className="flex items-center gap-2">
            <span className="w-3 h-3 rounded-full bg-blue-500" />
            <span>{agents.length} Agents Online</span>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
