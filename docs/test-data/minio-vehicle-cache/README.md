# MinIO vehicle cache test data

These fixtures are ready-to-upload S3 cache objects for the vehicle data fallback layer.

The backend reads one JSON object per normalized plate:

```text
<S3_CACHE_PREFIX><normalized-plate>.json
```

With the default local configuration, the bucket is `iviss-vehicle-cache` and the prefix is `vehicle-cache/`.

## Object format

Each file matches the Rust `CachedEntry` shape used by `vehicle_data_cache.rs`:

```json
{
  "data": {
    "plate_number": "LT 893 DK",
    "confidence": 1.0,
    "identification_mode": "manual",
    "vehicle": {},
    "status_results": {}
  },
  "cached_at": "2026-06-22T00:00:00Z"
}
```

Do not upload the external API raw payload shape (`{ "data": "<font ...>" }`) for this fallback test. The S3 cache deserializes already-parsed `VehicleSearchResult` data.

## Fixture keys

Upload these files with these exact object keys:

```text
vehicle-cache/LT893DK.json
vehicle-cache/CE128BC.json
vehicle-cache/LT3334W.json
vehicle-cache/SN1234.json
vehicle-cache/1234567.json
vehicle-cache/EN1234X.json
vehicle-cache/RT123456.json
vehicle-cache/CD34444.json
vehicle-cache/NW777AB.json
vehicle-cache/OU4567C.json
```

The application validates and normalizes agent input before cache lookup. For example, `LT 893 DK`, `lt-893-dk`, and `LT893DK` all resolve to the object key `vehicle-cache/LT893DK.json`.

## Manual MinIO upload

From the repository root, after configuring your `mc` alias:

```bash
mc alias set iviss-minio http://localhost:9000 "$MINIO_ROOT_USER" "$MINIO_ROOT_PASSWORD"
mc cp docs/test-data/minio-vehicle-cache/vehicle-cache/LT893DK.json iviss-minio/iviss-vehicle-cache/vehicle-cache/LT893DK.json
mc cp docs/test-data/minio-vehicle-cache/vehicle-cache/CE128BC.json iviss-minio/iviss-vehicle-cache/vehicle-cache/CE128BC.json
mc cp docs/test-data/minio-vehicle-cache/vehicle-cache/LT3334W.json iviss-minio/iviss-vehicle-cache/vehicle-cache/LT3334W.json
mc cp docs/test-data/minio-vehicle-cache/vehicle-cache/SN1234.json iviss-minio/iviss-vehicle-cache/vehicle-cache/SN1234.json
mc cp docs/test-data/minio-vehicle-cache/vehicle-cache/1234567.json iviss-minio/iviss-vehicle-cache/vehicle-cache/1234567.json
mc cp docs/test-data/minio-vehicle-cache/vehicle-cache/EN1234X.json iviss-minio/iviss-vehicle-cache/vehicle-cache/EN1234X.json
mc cp docs/test-data/minio-vehicle-cache/vehicle-cache/RT123456.json iviss-minio/iviss-vehicle-cache/vehicle-cache/RT123456.json
mc cp docs/test-data/minio-vehicle-cache/vehicle-cache/CD34444.json iviss-minio/iviss-vehicle-cache/vehicle-cache/CD34444.json
mc cp docs/test-data/minio-vehicle-cache/vehicle-cache/NW777AB.json iviss-minio/iviss-vehicle-cache/vehicle-cache/NW777AB.json
mc cp docs/test-data/minio-vehicle-cache/vehicle-cache/OU4567C.json iviss-minio/iviss-vehicle-cache/vehicle-cache/OU4567C.json
```

For the fallback to be active, the backend must run with S3 cache enabled and MinIO credentials configured, for example:

```text
S3_CACHE_ENABLED=true
S3_CACHE_BUCKET=iviss-vehicle-cache
S3_CACHE_PREFIX=vehicle-cache/
S3_CACHE_ENDPOINT_URL=http://minio:9000
S3_CACHE_FORCE_PATH_STYLE=true
AWS_ACCESS_KEY_ID=<minio user>
AWS_SECRET_ACCESS_KEY=<minio password>
```

