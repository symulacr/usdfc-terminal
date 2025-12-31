# USDFC Terminal - LayerZero Scan Style Redesign Plan

## Objective
Complete visual redesign to match LayerZero Scan aesthetic. Pure monochrome UI chrome with color reserved exclusively for data visualization. Zero decorative color in interface elements.

---

## 1. Analyze Current vs Target Design

**Audit current styles.css for:**
- All color variables (--accent-cyan, --accent-green, etc.)
- All border-radius values
- All gradient definitions
- All box-shadow definitions
- All colored border definitions

**Map each to LayerZero equivalent:**

| Current Element | Current Style | Target Style |
|-----------------|---------------|--------------|
| Background | #0a0a0a | #000000 pure black |
| Cards | Colored borders, rounded | No border or 1px #222, 0px radius |
| Buttons | Colored, rounded | White border on black, 0px radius |
| Text primary | White | #ffffff |
| Text secondary | Gray variations | #888888 |
| Text muted | Darker gray | #555555 |
| Accent elements | Cyan/green/etc | White only |
| Status success | Green | Green (data only) |
| Status danger | Red | Keep for data context |

---

## 2. Color Budget Rules

**ZERO color allowed in:**
- Navigation/header/sidebar
- Buttons and form inputs
- Card borders and backgrounds
- Table headers and rows
- Icons (all white/gray)
- Modal/dropdown chrome
- Loading states
- Tooltips chrome

**Color ONLY allowed in:**
- Chart bars, lines, areas (multi-color for data series)
- Heatmap cells (grayscale gradient acceptable)
- Status badges (Delivered=green, Failed=red)
- Change indicators (+X% green, -X% red)
- Sparklines in metric cards
- Network graph nodes/edges (for differentiation)
- Sankey flow paths (for flow differentiation)

---

## 3. Component-by-Component Plan

### Header (src/components/header.rs + styles.css)
- Remove any accent colors
- Logo: white or grayscale only
- Nav links: white text, no colored hover states
- Search bar: black background, white border, white text
- Add Ctrl+K shortcut indicator like LayerZero

### Sidebar (src/components/sidebar.rs)
- Active state: white text, no colored indicator
- Hover: subtle white/gray background shift
- Icons: all white, no color coding
- Remove any colored badges or indicators

### Metric Cards (src/components/metric_card.rs)
- Background: #111111 or transparent
- Border: none or 1px #222222
- Border-radius: 0px
- Title: #888888 gray
- Value: #ffffff white
- Add mini sparkline chart (this CAN have subtle color)
- Info icon: white, not colored

### Data Tables (src/components/data_table.rs)
- Header row: #111111 background
- Border: 1px #222222
- No zebra striping or colored rows
- Hover: subtle #1a1a1a background
- Sort indicators: white arrows
- Exception: Change column can show red/green percentages

### Buttons (styles.css .btn classes)
- Primary: transparent bg, 1px white border, white text
- Secondary: transparent bg, 1px #444 border, gray text
- Hover: white bg, black text (invert)
- Border-radius: 0px on all
- No colored variants

### Form Inputs
- Background: transparent or #0a0a0a
- Border: 1px #333333
- Focus: 1px white border (no glow)
- Border-radius: 0px
- Placeholder: #555555

### Footer (src/components/footer.rs)
- Pure monochrome
- Network status: text only, no colored dot
- Block number: white text

---

## 4. Charts Color Palette (ONLY place for color)

**Define data visualization palette:**
```css
--chart-1: #00d4ff;  /* Cyan - primary series */
--chart-2: #a855f7;  /* Purple */
--chart-3: #22c55e;  /* Green */
--chart-4: #f59e0b;  /* Amber/Yellow */
--chart-5: #ec4899;  /* Pink */
--chart-6: #6366f1;  /* Indigo */
--chart-7: #14b8a6;  /* Teal */
--chart-8: #f97316;  /* Orange */
--chart-9: #8b5cf6;  /* Violet */
--chart-10: #64748b; /* Slate for "Others" */
```

**Apply to:**
- Stacked bar charts (like LayerZero Messages chart)
- Area charts (like LayerZero Applications chart)
- Network graph nodes
- Sankey flow paths
- Pie/donut charts if any

**Grayscale for heatmaps:**
```css
--heat-0: #1a1a1a;
--heat-1: #333333;
--heat-2: #4d4d4d;
--heat-3: #666666;
--heat-4: #808080;
--heat-5: #999999;
--heat-6: #b3b3b3;
--heat-7: #cccccc;
--heat-8: #e6e6e6;
--heat-9: #ffffff;
```

---

## 5. Border Radius Removal

**Find and replace ALL instances:**
```css
/* Search for these patterns */
border-radius: 4px;   → border-radius: 0;
border-radius: 8px;   → border-radius: 0;
border-radius: 12px;  → border-radius: 0;
border-radius: 16px;  → border-radius: 0;
border-radius: 50%;   → border-radius: 0; /* except gauge */
border-radius: 9999px; → border-radius: 0;
```

**Files to scan:**
- styles.css (primary)
- Any inline styles in .rs component files
- Check for rounded-* Tailwind classes if used

**Exception:**
- Gauge component needle pivot can remain circular

---

## 6. Shadow and Gradient Removal

**Remove all box-shadows:**
```css
box-shadow: none;
/* Remove any rgba() shadow values */
```

**Remove all gradients:**
```css
/* Replace */
background: linear-gradient(...);
/* With */
background: #111111; /* solid color */
```

**Remove glows:**
```css
/* Remove */
box-shadow: 0 0 20px rgba(0, 212, 255, 0.3);
text-shadow: 0 0 10px ...;
```

---

## 7. Page-Specific Plans

### Dashboard (dashboard.rs)
- Metric cards: monochrome chrome, colored sparklines only
- Recent transactions table: pure monochrome
- TCR gauge: keep colored zones (data visualization)
- Activity chart: colored series lines

### Transactions (transactions.rs)
- Search bar: white border, no color
- Results table: monochrome
- Status column: green/red badges (data context)
- Hash/address links: white with underline on hover

### Collateral (collateral.rs)
- Trove table: monochrome
- Health indicators: colored (Critical=red, Safe=green) - this is data
- ICR distribution chart: colored bars

### Network (network.rs)
- UI controls: monochrome
- Graph visualization: colored nodes by type
- This is where color is appropriate

### Sankey (sankey.rs)
- UI controls: monochrome
- Flow paths: colored by source/category
- This is where color is appropriate

### Stability (stability.rs)
- Stats cards: monochrome with sparklines
- Pool chart: colored area fill

---

## 8. Typography Standardization

**Match LayerZero type scale:**
```css
--font-family: 'Inter', -apple-system, sans-serif;
--text-xs: 11px;
--text-sm: 13px;
--text-base: 14px;
--text-lg: 16px;
--text-xl: 20px;
--text-2xl: 24px;
--text-3xl: 32px;

/* Weights */
--font-normal: 400;
--font-medium: 500;
--font-semibold: 600;
```

**Apply:**
- Page titles: text-2xl, font-semibold, white
- Section headers: text-lg, font-medium, white
- Card titles: text-sm, font-normal, #888888
- Card values: text-xl, font-semibold, white
- Table headers: text-xs, font-medium, #888888, uppercase
- Body text: text-sm, font-normal, #888888

---

## 9. Interactive States (Monochrome Only)

**Hover states:**
```css
/* Links */
a:hover { text-decoration: underline; }

/* Buttons */
.btn:hover {
    background: #ffffff;
    color: #000000;
}

/* Table rows */
tr:hover { background: #1a1a1a; }

/* Cards (if clickable) */
.card:hover { border-color: #444444; }
```

**Focus states:**
```css
*:focus-visible {
    outline: 1px solid #ffffff;
    outline-offset: 2px;
}
```

**Active states:**
```css
.nav-item.active {
    color: #ffffff;
    /* No colored indicator */
}
```

---

## 10. Specific CSS Variable Replacement

**Create new monochrome palette:**
```css
:root {
    /* Backgrounds */
    --bg-primary: #000000;
    --bg-secondary: #0a0a0a;
    --bg-tertiary: #111111;
    --bg-elevated: #1a1a1a;

    /* Text */
    --text-primary: #ffffff;
    --text-secondary: #888888;
    --text-muted: #555555;
    --text-disabled: #333333;

    /* Borders */
    --border-subtle: #1a1a1a;
    --border-default: #222222;
    --border-strong: #333333;
    --border-focus: #ffffff;

    /* UI Accents - ALL WHITE/GRAY */
    --accent-primary: #ffffff;
    --accent-secondary: #888888;

    /* Data Colors - ONLY for visualizations */
    --data-positive: #22c55e;
    --data-negative: #ef4444;
    --data-neutral: #888888;

    /* Chart palette */
    --chart-1: #00d4ff;
    --chart-2: #a855f7;
    --chart-3: #22c55e;
    --chart-4: #f59e0b;
    --chart-5: #ec4899;
    --chart-6: #6366f1;
    --chart-7: #14b8a6;
    --chart-8: #f97316;
    --chart-9: #8b5cf6;
    --chart-10: #64748b;
}
```

---

## 11. Implementation Verification Checklist

After changes, verify each page:

```
□ Header: No color except logo (if brand requires)
□ Sidebar: Pure white/gray icons and text
□ All cards: 0px radius, no colored borders
□ All buttons: White/gray only, sharp corners
□ All inputs: Sharp corners, white focus ring
□ All tables: Monochrome except data indicators
□ Charts: Colorful (this is correct)
□ Status badges: Colored (this is correct - data context)
□ Change %: Red/green (this is correct - data context)
□ No shadows anywhere
□ No gradients anywhere
□ No glowing effects
```

---

## 12. File Change Summary

**Primary files to modify:**
1. `src/styles.css` - Complete color/radius overhaul
2. `src/components/header.rs` - Remove colored elements
3. `src/components/sidebar.rs` - Monochrome nav items
4. `src/components/metric_card.rs` - Add sparkline, remove color
5. `src/components/charts.rs` - Keep/enhance colors (data viz)
6. `src/components/gauge.rs` - Keep colored zones (data viz)
7. `src/components/data_table.rs` - Monochrome except data columns
8. All 16 page files - Remove inline colored styles

**Do not modify chart color logic - that stays colorful**

---

## 13. CSS Classes to Update

**Classes that need color removal:**
```css
.metric-value.cyan    → .metric-value (white)
.metric-value.green   → .metric-value (white)
.metric-value.yellow  → .metric-value (white)
.metric-value.purple  → .metric-value (white)
.metric-value.red     → .metric-value (white)

.card-header (colored borders) → border: none or 1px #222

.sidebar-link.active (cyan indicator) → white text only

.btn-primary (cyan bg) → transparent + white border
.btn-secondary (gray) → transparent + #444 border

.input:focus (cyan glow) → 1px white border
```

**Data-Context Color Classes to KEEP:**
```css
/* Status badges */
.status-badge.success { color: var(--data-positive); }
.status-badge.danger { color: var(--data-negative); }
.status-badge.warning { color: #f59e0b; }

/* Transaction types */
.type-badge.mint { background: var(--chart-1); }
.type-badge.burn { background: var(--data-negative); }
.type-badge.transfer { background: #666; }

/* Change indicators */
.change-positive { color: var(--data-positive); }
.change-negative { color: var(--data-negative); }

/* Chart elements - all colored */
.chart-* { /* keep all colors */ }
.gauge-zone-* { /* keep all colors */ }
```

---

## 14. Verification Commands

After implementation, run this search to verify no stray colors:

```bash
# Find any remaining accent colors in UI context
grep -rn "accent-cyan\|accent-green\|accent-yellow" src/
grep -rn "#00d4ff\|#00ff88\|#ffd000" src/
grep -rn "color:.*var(--accent" src/pages/
```

Expected results: Only in chart/gauge/data-badge contexts.

---

## Execution Note

This is a PLANNING document only. No code changes yet.
Review this plan, confirm design direction, then execute section by section.
Start with styles.css variable replacement, then component updates, then page cleanup.
