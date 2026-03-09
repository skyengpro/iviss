IVISS Overview

The IVISS (Integrated Vehicle Inspection and Surveillance System) is a full-stack platform designed for government agencies to perform roadside vehicle inspections, maintain a centralized vehicle registry, manage enforcement actions, and streamline gray-card submissions.

Architecture
- Backend: PostgreSQL database with a multi-tenant data model; REST/OpenAPI API layer.
- Frontend: React/TypeScript SPA for back-office and mobile-oriented interfaces.
- Documentation and APIs are exposed via an OpenAPI specification located in frontend/openapi-rq.

Core data model (high level)
- Organizations: multi-tenant boundary for different agencies
- Users: agents, managers, admins; role-based access control
- Vehicles: master registry with plate_number, VIN, brand, model, year, color, etc.
- Vehicle Owners: ownership history for vehicles
- Vehicle Statuses: cached external data (insurance, technical status, stolen flag, last_updated)
- Control Records: individual vehicle checks performed by agents
- Control Actions: enforcement actions tied to control records (citation, impound, flag, warning)
- Pending Submissions: gray-card submissions for vehicles not yet in registry, awaiting back-office processing

Workflows
- Vehicle check: agent creates a control record; data stored with timestamp, geolocation, and results
- Enforcement: control actions linked to control records
- Gray-card submission: new vehicles enter registry via back-office processing of pending submissions

Data considerations
- Multi-tenant data isolation by organization
- Audit trails with created_at/updated_at and soft-delete where applicable
- Retention and archival policies to balance compliance with storage costs

Getting started
- Prerequisites: Docker and Docker Compose
- Run locally: docker-compose up -d (from project root)
- See docs/docker_setup.md for environment setup, and docs/schema.md for the ERD

References
- docs/schema.md for the data model and relationships
- frontend/openapi-rq for the API surface