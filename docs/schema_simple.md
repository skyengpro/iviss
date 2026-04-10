# Database Schema

This document explains the database tables and what they store.

## Main Tables

### organizations
Government agencies that use the system (police brigades, customs offices, border control).

Key fields:
- name: Organization name
- type: police, customs, or border_control
- region: Geographic area

### users
All user accounts (agents, managers, admins).

Key fields:
- organization_id: Which organization they belong to
- username, email: Login credentials
- role: admin, agent, or manager
- badge_id: Official badge number
- is_active: Account status

### vehicles
The central vehicle registry.

Key fields:
- plate_number: License plate (unique)
- chassis_number: VIN
- brand, model, year, color: Vehicle details

### vehicle_owners
Links vehicles to their legal owners. One vehicle can have multiple owners.

Key fields:
- vehicle_id: Which vehicle
- name, address, national_id: Owner details

### vehicle_statuses
Cached compliance information from external systems.

Key fields:
- vehicle_id: Which vehicle
- insurance_status, insurance_expiry: Insurance info
- technical_status, technical_expiry: Technical inspection info
- stolen_status: Whether reported stolen
- last_updated: When cache was refreshed

### control_records
Every vehicle check performed by agents.

Key fields:
- agent_id: Who performed the check
- organization_id: Which organization
- plate_number: Plate that was checked
- timestamp: When it happened
- latitude, longitude: GPS location
- identification_mode: manual, photo, or live
- overall_status: valid, warning, or critical
- results_json: Detailed breakdown of all checks

### control_actions
Specific enforcement actions taken during controls.

Key fields:
- control_id: Which control record
- action_type: citation, impound, flag, or warning
- description: Details of the action

### pending_submissions
Gray card (vehicle registration) documents submitted by agents for admin review.

Key fields:
- agent_id: Who submitted it
- plate_number: Plate from the document
- front_image_url, back_image_url: Photos of the gray card
- status: pending, approved, or rejected
- reviewed_by: Admin who processed it

## How Tables Connect

- Each user belongs to one organization
- Each control record is created by one user (agent)
- Each control record can have multiple actions
- Each vehicle can have multiple owners
- Each vehicle has one status cache record
- Each pending submission is created by one agent and reviewed by one admin

## Data Isolation

Data is isolated by organization_id. Agents and managers can only see data from their own organization. Admins can see all organizations.
