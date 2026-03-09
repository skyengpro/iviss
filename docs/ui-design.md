# IVISS UI Design - Dashboard Visual Specification

This document defines the unique visual identity and aesthetic rules for the IVISS Backoffice Dashboard, ensuring a persistent, premium, and "government-grade" experience.

## Visual Identity: "Technocratic Authority"

The IVISS aesthetic combines traditional authority (stable deep navies) with cutting-edge technocracy (vibrant teals and precise data visualizations).

### 1. Color Palette: "Authority & Precision"

-   **Primary (Authority)**: `hsl(222 47% 20%)` – Deep, stable, and commanding.
-   **Accent (Modernity)**: `hsl(186 72% 38%)` – Energetic, high-tech, and precise.
-   **Surface (Minimalism)**: `hsl(220 20% 97%)` – Clean, airy, and focused on content.
-   **Status (Vibrancy)**:
    -   Valid: `hsl(142 71% 45%)` (Vibrant Emerald)
    -   Warning: `hsl(38 92% 50%)` (Deep Amber)
    -   Critical: `hsl(0 84% 60%)` (Alert Crimson)

### 2. Design Tokens: "The Premium Layer"

| Element | Rule |
| :--- | :--- |
| **Glassmorphism** | Use `backdrop-blur(12px)` with `bg-background/80` for elevated ephemeral elements. |
| **Gradients** | Prefer subtle diagonal gradients (135deg) over flat colors for primary and status cards. |
| **Borders** | Use `hsl(var(--border) / 0.5)` for very subtle container boundaries. |
| **Shadows** | Use large, soft shadows (`--shadow-xl`) for main cards to create depth without clutter. |
| **Typography** | Inter font-family. Bold headers (`font-bold`) and mono-spaced tracking IDs for a technical feel. |

### 3. Interactive Mechanics: "Micro-Gestalt"

-   **Entrance**: All dashboard components enter with `animate-slide-up` and a staggered delay.
-   **Hover States**: Cards subtly lift (transform: translateY(-2px)) and their shadows deepen (`shadow-card-hover`).
-   **Pulse**: Active markers and "auto-updating" indicators use the `animate-pulse-status`.
-   **Transitions**: All visual changes use `duration-300 ease-in-out` for a smooth, high-quality flow.

## Guidelines for New Components

-   **Hierarchy**: Keep the Primary Action or Most Important Stat at the top-left (following F-pattern).
-   **Density**: Maintain sufficient whitespace. Never crowd more than 4 items in a row on large screens.
-   **Visual Balance**: Balance the heavy "Authority Navy" with lighter "Slate" and "White" surfaces to avoid a gloomy interface.
-   **Consistency**: Always use the predefined `@layer utility` animations instead of ad-hoc transitions.
