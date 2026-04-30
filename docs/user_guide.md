# IVISS User Guide

**Welcome to IVISS** — Your complete platform for vehicle identification, compliance checks, and field operations management.

---

## Who This Is For

IVISS is designed for:

- **Law enforcement agencies** conducting roadside vehicle inspections and enforcement
- **Government regulatory bodies** managing vehicle compliance and registration
- **Field agents** who need quick, reliable vehicle identification on the go
- **Supervisors and administrators** who coordinate field operations and manage teams
- **Organizations** responsible for public safety and regulatory compliance

If your work involves checking vehicle status, verifying compliance, or managing field operations, IVISS streamlines these tasks with real-time data and mobile-first tools.

---

## What IVISS Does

IVISS helps you:

- **Identify vehicles instantly** using license plate recognition (scan or manual entry)
- **Check compliance status** for insurance, customs clearance, technical inspection, and wanted vehicles
- **Record enforcement actions** like citations, warnings, impounds, and flags
- **Track all vehicle checks** with complete audit trails including location and timestamps
- **Manage field teams** with role-based access and device security
- **Generate reports** on control activities, agent performance, and vehicle statistics
- **Work offline** with Progressive Web App (PWA) technology that syncs when you're back online

---

## Core Features at a Glance

| Feature                          | What It Does                                                                 |
| -------------------------------- | ---------------------------------------------------------------------------- |
| **License Plate Recognition**    | Scan plates with your camera or enter them manually                          |
| **Real-Time Compliance Checks**  | Instantly verify insurance, customs, inspection, and wanted status           |
| **Control History**              | Complete record of all vehicle checks with timestamps and locations          |
| **Alert System**                 | Immediate notifications for flagged vehicles (stolen, unregistered, etc.)    |
| **Mobile-First Design**          | Optimized for smartphones and tablets used in the field                      |
| **Progressive Web App (PWA)**    | Install on any device, works offline, updates automatically                  |
| **Multi-Organization Support**   | Separate data and settings for different agencies                            |
| **Role-Based Access Control**    | Different permissions for Super Admins, Admins, Supervisors, and Agents      |
| **Enforcement Actions**          | Record citations, impounds, warnings, and flags directly from the field      |
| **Report Generation**            | Export control summaries, agent performance, and statistics in CSV/PDF/Excel |
| **Secure Device Binding**        | Each agent's device is cryptographically linked to their account             |
| **Daily Shift Management**       | Agents verify their identity each shift with SMS codes                       |

---

## Getting Started: Your First Steps

### For Administrators

#### 1. Access the Back-Office

**Prerequisites:**
- Admin account credentials (email and password)
- Web browser (Chrome, Firefox, Safari, or Edge recommended)
- Internet connection

**Steps:**

1. Open your web browser and navigate to the IVISS back-office URL (provided by your system administrator)
2. Click **Admin Login** on the welcome screen
3. Enter your **email address** and **password**
4. Click **Sign In**
5. You'll be redirected to the **Back-Office Dashboard**

**What you'll see:**
- Dashboard with system statistics
- Navigation menu on the left with options for Users, Organizations, Controls, Reports, and Settings

---

#### 2. Create Your First Organization

**Prerequisites:**
- Super Admin role
- Organization details (name, contact information)

**Steps:**

1. From the back-office dashboard, click **Organizations** in the left sidebar
2. Click the **Add Organization** button (top right)
3. Fill in the organization details:
   - **Name**: The official name of the agency or department
   - **Contact Email**: Primary contact for the organization
   - **Phone Number**: Main contact number
   - **Address**: Physical location (optional)
4. Click **Create Organization**
5. The new organization appears in your organizations list

**What happens next:**
- The organization is now active in the system
- You can now create admin users for this organization
- Each organization's data is completely isolated from others

---

#### 3. Create Admin Users

**Prerequisites:**
- Admin or Super Admin role
- User details (email, name, role)

**Steps:**

1. Click **Users** in the left sidebar
2. Click **Add User** button
3. Fill in the user information:
   - **Email**: User's email address (used for login)
   - **First Name** and **Last Name**
   - **Role**: Select **Admin** from the dropdown
   - **Organization**: Select the organization this admin will manage
   - **Password**: Create a secure password (minimum 8 characters)
4. Click **Create User**
5. The new admin receives their credentials and can now log in

**Important notes:**
- Admins can only manage users within their own organization
- Super Admins can manage users across all organizations
- Passwords should be changed on first login (see Settings)

---

#### 4. Provision Field Agents

**Prerequisites:**
- Admin role in the organization
- Agent details (name, phone number, badge ID)
- Agent's mobile device ready

**Steps:**

1. Click **Users** in the left sidebar
2. Click **Add User** button
3. Fill in the agent information:
   - **First Name** and **Last Name**
   - **Phone Number**: Must be accurate (used for SMS verification)
   - **Badge ID**: Agent's official identification number
   - **Role**: Select **Agent** from the dropdown
   - **Organization**: Your organization (auto-selected)
4. Click **Create User**
5. The system generates an **activation code** and sends it via SMS to the agent's phone
6. Provide the agent with instructions to activate their device (see Agent section below)

**What happens next:**
- Agent receives an SMS with their activation code
- Agent can now activate their device and start working
- You can track the agent's activation status in the Users list

---

### For Field Agents

#### 1. Activate Your Device (First Time Only)

**Prerequisites:**
- Activation code received via SMS from your administrator
- Mobile device (smartphone or tablet)
- Internet connection
- Your badge ID

**Steps:**

1. Open your web browser on your mobile device
2. Navigate to the IVISS mobile app URL (provided by your administrator)
3. On the welcome screen, tap **Activate Device**
4. Enter your **phone number** (the one registered with your administrator)
5. Enter the **activation code** from your SMS
6. Enter your **badge ID**
7. Tap **Activate**
8. Your device is now registered and you're logged in

**What happens:**
- Your device generates a unique cryptographic identity
- This identity is permanently linked to your account
- You receive access tokens that allow you to use the app
- Your device status is set to **ACTIVE**

**Important:**
- This activation only happens once per device
- If you clear your browser data, you'll need to activate again
- Keep your device secure — it's your key to the system

---

#### 2. Daily Login (Start of Each Shift)

**Prerequisites:**
- Activated device
- Phone number registered in the system
- Internet connection

**Steps:**

1. Open the IVISS app on your device
2. If your previous shift has ended, you'll see the **Daily Login** screen
3. Enter your **phone number**
4. Tap **Request Code**
5. Wait for the SMS with your daily code (arrives within 1 minute)
6. Enter the **6-digit code** from the SMS
7. Tap **Verify**
8. You're now logged in for your shift

**What you'll see:**
- The main vehicle search screen
- Your shift end time displayed at the top
- Access to all field operations features

**Important notes:**
- Daily codes expire after 5 minutes
- You can request a new code if the first one expires
- Maximum 3 code requests per 10 minutes (prevents abuse)
- Your shift automatically ends at the scheduled time
- You'll need to log in again for your next shift

---

#### 3. Check a Vehicle

**Prerequisites:**
- Active shift (logged in)
- Vehicle license plate number

**Steps:**

##### Option A: Live Scan Mode (Real-Time Recognition)

1. From the main screen, tap the **Camera** icon
2. Allow camera access if prompted
3. Select **Live** mode at the bottom of the screen
4. Tap **Start Live Scan**
5. Point your camera at the vehicle's license plate
6. Keep the plate centered in the viewfinder frame
7. The app continuously scans and detects the plate automatically
8. When a valid plate is detected, it appears on screen
9. Review the detected plate number
10. Tap **Confirm** to search the vehicle

**Best for:** Quick checks when the vehicle is stationary and the plate is clearly visible.

##### Option B: Photo Mode (Single Capture with Quality Check)

1. From the main screen, tap the **Camera** icon
2. Allow camera access if prompted
3. Select **Photo** mode at the bottom of the screen
4. Point your camera at the vehicle's license plate
5. Keep the plate centered in the viewfinder frame
6. Tap the **white capture button** to take a photo
7. The app automatically:
   - Assesses image quality (brightness, blur, contrast)
   - Crops to the viewfinder area for better accuracy
   - Sends the image to the OCR engine
8. Review the detected plate number
9. If needed, tap **Edit** to correct any characters
10. Tap **Confirm** to search the vehicle

**Best for:** Difficult lighting conditions, moving vehicles, or when you need a single high-quality capture.

##### Option C: Enter Plate Manually

1. From the main screen, tap the **Plate Number** field
2. Type the license plate number
   - The app automatically formats as you type
   - Converts to uppercase
   - Adds spaces where needed
3. When the plate is valid, the **Search** button activates
4. Tap **Search**

**Supported plate formats:**
- Standard Regional: `CE 1234 A` or `LT 123 AB`
- Police: `SN 1234`
- Military: `1234567` (7 digits)
- Government: `EN1234X`
- Postal: `RT123456`
- Diplomatic: `CD 12 345`

**What happens next:**
- The system searches the national vehicle database
- Compliance checks run automatically (insurance, customs, inspection, wanted status)
- Results appear within 2-3 seconds
- A control record is created with your location and timestamp

---

#### 4. Review Vehicle Information

**What you'll see after a search:**

**Vehicle Details:**
- License plate number
- Make, model, and year
- Color
- VIN (chassis number)
- Owner information

**Compliance Status:**
- **Insurance**: Valid/Invalid/Expired with expiry date
- **Customs Clearance**: Cleared/Not Cleared with date
- **Technical Inspection**: Valid/Invalid with next due date
- **Wanted Status**: Flagged/Clear

**Status Indicators:**
- ✅ **Green**: Compliant
- ⚠️ **Yellow**: Warning (expiring soon)
- ❌ **Red**: Non-compliant or flagged

**Alert Notifications:**
- If a vehicle is **wanted** or **stolen**, you'll see a prominent red alert
- If insurance or inspection is **expired**, you'll see a warning
- Follow your organization's procedures for flagged vehicles

---

#### 5. Record an Enforcement Action

**Prerequisites:**
- Completed vehicle check
- Reason for enforcement action

**Steps:**

1. After viewing vehicle details, scroll to **Enforcement Actions**
2. Tap **Add Action**
3. Select the **action type**:
   - **Citation**: Issue a ticket for a violation
   - **Warning**: Verbal or written warning
   - **Impound**: Vehicle seized
   - **Flag**: Mark for follow-up
4. Enter **details** or **notes** about the action
5. Add any **photos** if required (tap Camera icon)
6. Tap **Save Action**

**What happens:**
- The action is recorded with timestamp and your agent ID
- The action is linked to the control record
- Your supervisor can review all actions in the back-office
- The vehicle owner may be notified (depending on configuration)

**Best practices:**
- Be specific in your notes
- Include relevant details (location, circumstances, driver behavior)
- Take clear photos if documenting damage or violations
- Follow your organization's enforcement guidelines

---

#### 6. View Your Control History

**Steps:**

1. Tap the **Menu** icon (three lines, top left)
2. Select **My History**
3. You'll see a list of all your vehicle checks

**What you can do:**
- **Filter** by date range
- **Search** for specific plates
- **View details** of past controls
- **Review** enforcement actions you've taken

**Information shown:**
- Date and time of each check
- License plate number
- Vehicle make and model
- Compliance status at time of check
- Location where check was performed
- Any enforcement actions taken

---

### For Supervisors

Supervisors have access to both mobile field operations and back-office monitoring features.

#### 1. Monitor Agent Activity

**Prerequisites:**
- Supervisor role
- Access to back-office

**Steps:**

1. Log in to the back-office
2. Click **Controls** in the left sidebar
3. View the **Control Activity Dashboard**

**What you'll see:**
- Real-time map of agent locations and recent checks
- List of all controls performed by your team
- Statistics on checks per agent
- Compliance violation trends
- Enforcement actions taken

**Filtering options:**
- By date range
- By specific agent
- By vehicle status (compliant/non-compliant)
- By enforcement action type

---

#### 2. Generate Reports

**Prerequisites:**
- Supervisor or Admin role
- Access to back-office

**Steps:**

1. Click **Reports** in the left sidebar
2. Select a **report type**:
   - **Control Summary**: Overview of all vehicle checks
   - **Agent Performance**: Activity and productivity by agent
   - **Vehicle Status**: Compliance statistics
   - **Organization Statistics**: High-level metrics
3. Set the **date range**
4. Select any additional **filters** (agent, status, etc.)
5. Click **Generate Report**
6. Choose **export format**:
   - **CSV**: For spreadsheet analysis
   - **PDF**: For printing or sharing
   - **Excel**: For advanced data manipulation
7. Click **Export**

**Report contents:**
- Summary statistics
- Detailed data tables
- Charts and visualizations (PDF only)
- Timestamp and generated by information

---

## Common Workflows

### Workflow 1: Morning Shift Start

**For Agents:**

1. Arrive at your station or patrol area
2. Open the IVISS app on your device
3. Request your daily login code
4. Check your phone for the SMS code
5. Enter the code to start your shift
6. Verify your shift end time is correct
7. Begin patrol and vehicle checks

**Time required:** 2-3 minutes

---

### Workflow 2: Roadside Vehicle Check

**For Agents:**

1. Pull over a vehicle for inspection
2. Open IVISS app (already logged in)
3. Scan the license plate or enter it manually
4. Wait 2-3 seconds for results
5. Review vehicle details and compliance status
6. Check for alerts (wanted, stolen, expired documents)
7. If non-compliant, record an enforcement action
8. If compliant, inform the driver and let them proceed
9. Move to next vehicle

**Time required:** 1-2 minutes per vehicle

---

### Workflow 3: Handling a Flagged Vehicle

**For Agents:**

1. Search for the vehicle (scan or manual entry)
2. See **RED ALERT** for wanted/stolen status
3. **Do not approach alone** — follow safety protocols
4. Call for backup if required
5. Record the sighting with location details
6. Take photos if safe to do so
7. Follow your organization's procedures for wanted vehicles
8. Complete the control record with detailed notes

**Important:** Your safety comes first. Never put yourself at risk.

---

### Workflow 4: End of Shift

**For Agents:**

1. Complete your final vehicle checks
2. Return to your station
3. The app automatically logs you out at shift end time
4. Review any pending actions or reports
5. Close the app
6. Your device returns to standby mode

**For Supervisors:**

1. Review the day's control activity
2. Check for any flagged vehicles or issues
3. Generate daily summary report
4. Follow up on any enforcement actions
5. Prepare briefing for next shift

---

### Workflow 5: Adding a New Agent

**For Admins:**

1. Collect agent information (name, phone, badge ID)
2. Log in to back-office
3. Navigate to Users → Add User
4. Enter agent details and create account
5. System sends activation code via SMS
6. Provide agent with device activation instructions
7. Verify agent successfully activates their device
8. Assign agent to supervisor (if applicable)
9. Brief agent on procedures and expectations

**Time required:** 5-10 minutes

---

## Troubleshooting Common Issues

### Issue: "Activation code not received"

**Possible causes:**
- Wrong phone number entered
- SMS delay from carrier
- Phone has no signal

**Solutions:**
1. Verify the phone number is correct
2. Wait 2-3 minutes for SMS delivery
3. Check phone signal strength
4. Request a new code (wait 10 minutes between requests)
5. Contact your administrator if problem persists

---

### Issue: "Invalid activation code"

**Possible causes:**
- Code expired (5 minutes)
- Code already used
- Typo in code entry

**Solutions:**
1. Request a new code
2. Enter the code carefully (6 digits)
3. Use the most recent code received
4. Contact your administrator if problem continues

---

### Issue: "Device suspended"

**Possible causes:**
- Administrator suspended your device
- Security concern
- Device reported lost or stolen

**Solutions:**
1. Contact your supervisor or administrator immediately
2. Do not attempt to bypass the suspension
3. Wait for administrator to restore access
4. You may need to re-activate your device

---

### Issue: "Cannot scan license plate"

**Possible causes:**
- Poor lighting conditions
- Dirty or damaged plate
- Camera not focused
- Plate format not recognized
- Image quality too low (blurry, too dark, too bright)

**Solutions:**
1. **Switch to Photo Mode** — provides quality feedback and better accuracy
2. Ensure good lighting (use flashlight if needed)
3. Clean the plate if dirty
4. Hold camera steady and wait for focus
5. In Photo Mode, follow the quality feedback messages:
   - "Image too blurry" → Hold camera steadier
   - "Image too dark" → Add more light or move closer
   - "Image too bright" → Reduce direct sunlight or adjust angle
6. Try manual entry instead
7. Ensure plate is in a supported format

---

### Issue: "Vehicle not found"

**Possible causes:**
- Vehicle not registered in national database
- Plate number entered incorrectly
- New vehicle not yet in system
- Foreign vehicle

**Solutions:**
1. Verify plate number is correct
2. Try re-entering or re-scanning
3. Check if plate format is valid
4. For unregistered vehicles, follow your organization's procedures
5. Record the incident in your notes

---

### Issue: "Compliance status shows 'Unknown'"

**Possible causes:**
- Partner API temporarily unavailable
- Network timeout
- Vehicle data incomplete

**Solutions:**
1. Wait 30 seconds and search again
2. Check your internet connection
3. Note the unknown status in your report
4. Follow up later or use alternative verification methods
5. Report persistent issues to your supervisor

---

## Security Best Practices

### For All Users

1. **Never share your credentials** with anyone
2. **Log out** when not using the system
3. **Report suspicious activity** immediately
4. **Keep your device secure** — use screen lock
5. **Don't write down passwords** — use a password manager
6. **Change your password** if you suspect it's been compromised
7. **Be aware of phishing** — verify URLs before entering credentials

### For Agents

1. **Keep your device with you** at all times during shift
2. **Don't let others use your device** for IVISS
3. **Report lost or stolen devices** immediately
4. **Clear browser data** if device is compromised
5. **Use strong device lock** (PIN, fingerprint, face ID)
6. **Don't share activation codes** or daily login codes
7. **Log out** if leaving device unattended

### For Administrators

1. **Review user access** regularly
2. **Suspend inactive accounts** promptly
3. **Monitor for unusual activity** in audit logs
4. **Use strong passwords** for admin accounts
5. **Enable two-factor authentication** if available
6. **Limit admin privileges** to necessary users only
7. **Keep contact information** up to date for all users

---

## Understanding Roles and Permissions

### Super Admin

**Can do:**
- Create and manage all organizations
- Create and manage all users across organizations
- View all system data and audit logs
- Configure system-wide settings
- Access all reports and analytics
- Suspend or restore any user or device

**Cannot do:**
- Perform field operations (agent functions)

**Typical users:** System administrators, IT staff

---

### Admin

**Can do:**
- Manage users within their organization
- Create and assign agents
- View organization-specific data and reports
- Configure organization settings
- Suspend or restore users in their organization
- Generate reports for their organization

**Cannot do:**
- Access other organizations' data
- Create or modify organizations
- Perform field operations (agent functions)
- Change system-wide settings

**Typical users:** Department heads, operations managers

---

### Supervisor

**Can do:**
- View all controls performed by assigned agents
- Generate reports for their team
- Monitor agent activity in real-time
- Perform field operations (same as agents)
- Review enforcement actions

**Cannot do:**
- Create or manage users
- Change organization settings
- Access other supervisors' teams
- Suspend devices or users

**Typical users:** Team leaders, shift supervisors

---

### Agent

**Can do:**
- Perform vehicle checks (scan or manual entry)
- View vehicle details and compliance status
- Record enforcement actions
- View their own control history
- Update their profile and password

**Cannot do:**
- View other agents' activity
- Access back-office administration
- Create or manage users
- Generate system reports
- Change organization settings

**Typical users:** Field officers, patrol agents

---

## System Requirements

### For Mobile Agents

**Device:**
- Smartphone or tablet
- Android 8.0+ or iOS 12.0+
- Camera (for plate scanning)
- GPS (for location tracking)

**Browser:**
- Chrome 90+
- Safari 14+
- Firefox 88+
- Edge 90+

**Network:**
- 3G/4G/5G or WiFi connection
- Minimum 1 Mbps download speed

**Storage:**
- 50 MB free space for app data
- Additional space for photos (if taking enforcement photos)

---

### For Back-Office Users

**Device:**
- Desktop computer, laptop, or tablet
- Windows 10+, macOS 10.14+, or Linux

**Browser:**
- Chrome 90+ (recommended)
- Firefox 88+
- Safari 14+
- Edge 90+

**Network:**
- Broadband internet connection
- Minimum 5 Mbps download speed (for reports and dashboards)

**Display:**
- Minimum 1280x720 resolution
- 1920x1080 or higher recommended

---

## Getting Help

### In-App Help

- Look for the **Help** icon (question mark) in the app
- Tap for context-sensitive help on any screen
- Access user guides and tutorials

### Contact Your Administrator

For issues with:
- Account access or passwords
- Device activation or suspension
- User permissions or roles
- Organization settings

Contact your organization's IVISS administrator.

### Technical Support

For technical issues:
- System errors or bugs
- Performance problems
- Feature requests
- Integration questions

Contact IVISS technical support (contact information provided by your system administrator).

## Frequently Asked Questions

**Q: How long does device activation take?**
A: 2-3 minutes once you receive your activation code.

**Q: Can I use IVISS on multiple devices?**
A: Each device must be activated separately. Contact your administrator to register additional devices.

**Q: What happens if I lose my device?**
A: Report it to your administrator immediately. They will suspend the device to prevent unauthorized access.

**Q: Can I work offline?**
A: IVISS is a Progressive Web App with limited offline capabilities. You can view previously loaded data, but vehicle searches require an internet connection.

**Q: How long do daily login codes last?**
A: 5 minutes. Request a new code if yours expires.

**Q: Can I change my shift hours?**
A: Contact your administrator to adjust shift times.

**Q: What's the difference between Live Scan and Photo Mode?**
A: Live Scan continuously detects plates in real-time, while Photo Mode captures a single high-quality image with quality assessment and allows you to edit the result before searching.

**Q: Why does Photo Mode reject my image?**
A: Photo Mode includes quality checks for blur, brightness, and contrast. Follow the feedback messages to improve image quality (add light, hold steady, adjust angle).

**Q: Can I edit a detected plate number?**
A: Yes, in Photo Mode you can tap **Edit** to manually correct any misread characters before confirming the search.

**Q: What if a vehicle has multiple compliance issues?**
A: Record all issues in your enforcement action notes. You can add multiple actions to a single control record.

**Q: How do I update my profile information?**
A: Go to Settings → Profile in the app or back-office.

**Q: Can I delete a control record?**
A: No. All control records are permanent for audit purposes. Contact your administrator if you need to add corrections or notes.

**Q: How far back can I view my control history?**
A: All your controls are available indefinitely. Use date filters to find specific records.

---

## Glossary

**Agent**: A field user who performs vehicle checks and enforcement actions.

**Activation Code**: A one-time code sent via SMS to register a new device.

**Back-Office**: The web-based administrative interface for supervisors and admins.

**Compliance Status**: Whether a vehicle meets requirements for insurance, customs, inspection, etc.

**Control**: A vehicle check or inspection performed by an agent.

**Control Record**: The complete documentation of a vehicle check, including timestamp, location, and results.

**Daily Login**: The SMS-based verification required at the start of each shift.

**Device Binding**: The cryptographic link between an agent's account and their physical device.

**Enforcement Action**: A recorded action taken against a vehicle (citation, warning, impound, flag).

**Multi-Tenant**: System architecture that keeps each organization's data completely separate.

**OTP**: One-Time Password — a temporary code sent via SMS for authentication.

**PWA**: Progressive Web App — a web application that works like a native app with offline support.

**RBAC**: Role-Based Access Control — permissions system based on user roles.

**Shift**: The time period during which an agent is logged in and authorized to work.

**Super Admin**: System-wide administrator with access to all organizations.

**Wanted Status**: Indicates if a vehicle is reported stolen or flagged by authorities.

---

## Document Version

**Version:** 1.0
**Last Updated:** April 30, 2026
**Author:** IVISS Development Team

For the latest version of this guide, check the Help section in the IVISS back-office or contact your system administrator.

---

**Welcome to IVISS. We're here to make your work safer, faster, and more effective.**
