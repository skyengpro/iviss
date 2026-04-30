# Manual Plate Entry Documentation

This document outlines the supported license plate formats for manual entry in the IVISS mobile application. These formats are specifically tailored for vehicles registered in Cameroon.

## Supported Formats

The system currently supports the following seven license plate patterns:

### 1. Standard Regional (Long)
- **Format**: `(REGION) 1234 A`
- **Example**: `CE 1234 A`, `LT 5678 B`
- **Description**: Standard civilian plates with a 2-letter region code, 4 digits, and a single letter.
- **Valid Regions**: AD, CE, ES, EN, LT, NO, NW, OU, SU, SW.

### 2. Standard Regional (Short)
- **Format**: `(REGION) 123 AB`
- **Example**: `LT 123 AB`, `CE 999 ZZ`
- **Description**: Standard civilian plates with a 2-letter region code, 3 digits, and 2 letters.

### 3. Police (National Security)
- **Format**: `SN 1234`
- **Example**: `SN 1102`
- **Description**: Vehicles belonging to the Sûreté Nationale. Requires the `SN` prefix followed by a space and 4 digits.

### 4. Military Vehicles
- **Format**: `1234567`
- **Example**: `3123456`, `9008877`
- **Description**: A strict 7-digit numeric format used by various branches of the military.

### 5. Government / State Vehicles
- **Format**: `AB1234X`
- **Example**: `EN1234X`, `CA5678Y`
- **Description**: Specific institution codes (2 letters) followed by 4 digits and a series letter. No spaces required.

### 6. Postal Service
- **Format**: `RT123456`
- **Example**: `RT112233`
- **Description**: Vehicles for Post & Telecommunications. Starts with the `RT` prefix followed by 6 digits.

### 7. Diplomatic Plates
- **Format**: `CD 12 345` or `CD 123 456`
- **Example**: `CD 01 123`, `CD 155 999`
- **Description**: Diplomatic Corps vehicles. Starts with `CD` followed by spaces separating the country code and serial number.

---

## Technical Implementation

### Validation
Validation is handled by the `isValidPlate` utility in `PlateInput.tsx`. It uses a comprehensive regular expression that covers all the above cases.

> **Note:**
> The validation logic automatically trims leading and trailing spaces, making it more user-friendly during manual typing.

### Auto-Formatting
The `PlateInput` component provides "flexible formatting":
- Automatically converts characters to **UPPERCASE**.
- Collapses multiple spaces into a single space.
- Removes invalid special characters.
- Allows spaces to be typed naturally between segments.

### UI Feedback
- The **Search Button** in the mobile interface only enables when a valid format is detected.
- An error message is displayed if a user attempts to submit an incomplete or invalid plate number.
