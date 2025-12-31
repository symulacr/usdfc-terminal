# USDFC Analytics Terminal - Comprehensive UI/UX Audit (Phase 1)

## Objective
Deep audit of Rust/Leptos dashboard codebase to identify every UI/UX issue preventing market readiness. Examination of all 16 pages, components, layouts, and interactions.

---

## CRITICAL ISSUES (Breaks Functionality)

---

### ISSUE C-01: Panic-Prone Timestamp Parsing
```
FILE: src/pages/dashboard.rs
LINE: 197-201
ISSUE: Using deprecated NaiveDateTime::from_timestamp_opt with .expect() causes panic
SEVERITY: Critical
FIX: Use checked parsing with proper error handling
CODE:
  // Before
  fn format_timestamp(seconds: u64) -> String {
      let dt = chrono::NaiveDateTime::from_timestamp_opt(seconds as i64, 0)
          .expect("timestamp out of range");

  // After
  fn format_timestamp(seconds: u64) -> String {
      chrono::DateTime::from_timestamp(seconds as i64, 0)
          .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
          .unwrap_or_else(|| "Invalid time".to_string())
```
**Note:** This pattern is repeated in 6 files: `dashboard.rs:197`, `transactions.rs:193`, `collateral.rs:197`, `stability.rs:173`, `supply.rs:150`, `lending.rs:165`

---

### ISSUE C-02: Panic on Decimal Parse Failure
```
FILE: src/pages/collateral.rs
LINE: 94
ISSUE: .expect("decimal parse") will panic on malformed data from RPC
SEVERITY: Critical
FIX: Handle parse errors gracefully
CODE:
  // Before
  let icr: f64 = t.icr.to_string().parse().expect("decimal parse");

  // After
  let icr: f64 = t.icr.to_string().parse().unwrap_or(0.0);
```
**Note:** This pattern appears 20+ times across pages - all `parse().expect()` calls should use `unwrap_or_default()` or proper error handling.

---

### ISSUE C-03: Balance Parse Panic with Invalid Input
```
FILE: src/pages/transactions.rs
LINE: 209-217
ISSUE: format_balance() calls .expect() on user-provided balance string
SEVERITY: Critical
FIX: Return fallback value on parse failure
CODE:
  // Before
  fn format_balance(balance: &str) -> String {
      let val: f64 = balance.parse().expect("balance parse");

  // After
  fn format_balance(balance: &str) -> String {
      let val: f64 = balance.parse().unwrap_or(0.0);
```

---

## HIGH SEVERITY ISSUES (Significant UX Degradation)

---

### ISSUE H-01: No Loading Skeleton for Metric Cards
```
FILE: src/components/metric_card.rs
LINE: 1-56
ISSUE: MetricCard has no loading prop - parent must wrap with Suspense, creating layout shift
SEVERITY: High
FIX: Add isLoading prop with skeleton state using existing .skeleton CSS class
CODE:
  // Add prop
  #[prop(default = false)] is_loading: bool,

  // In view:
  {if is_loading {
      view! {
          <div class="metric-card">
              <div class="skeleton" style="height: 14px; width: 80px; margin-bottom: 8px;"></div>
              <div class="skeleton" style="height: 28px; width: 120px;"></div>
          </div>
      }.into_view()
  } else { /* current implementation */ }}
```

---

### ISSUE H-02: DataTable Has No Pagination
```
FILE: src/components/data_table.rs
LINE: 1-24
ISSUE: DataTable renders ALL rows with no pagination or virtual scroll
SEVERITY: High
FIX: Add pagination props (page, page_size, total) and controls
CODE:
  // Add props
  #[prop(default = 1)] page: usize,
  #[prop(default = 20)] page_size: usize,
  #[prop(optional)] on_page_change: Option<Callback<usize>>,

  // Slice data in render:
  let start = (page - 1) * page_size;
  let page_data = &data[start..std::cmp::min(start + page_size, data.len())];
```

---

### ISSUE H-03: DataTable Has No Sort Indicators
```
FILE: src/components/data_table.rs
LINE: 13-15
ISSUE: Table headers have no sorting functionality or visual indicators
SEVERITY: High
FIX: Add sortable column headers with ChevronUp/ChevronDown icons
CODE:
  // Add to th element:
  <th
      style="cursor: pointer;"
      on:click=move |_| on_sort(col_index)
  >
      {*h}
      {sort_icon(col_index, sort_state)}
  </th>
```

---

### ISSUE H-04: No Empty State Component
```
FILE: src/components/data_table.rs
LINE: 17-19
ISSUE: Empty data shows nothing - no visual feedback to user
SEVERITY: High
FIX: Add empty state message when data.is_empty()
CODE:
  // Add before tbody:
  {if data.is_empty() {
      view! {
          <div class="empty-state">
              <div class="empty-state-icon"><SearchIcon /></div>
              <div class="empty-state-title">"No data available"</div>
              <div class="empty-state-desc">"Try adjusting your filters or check back later."</div>
          </div>
      }
  } else { /* render table */ }}
```

---

### ISSUE H-05: Addresses Not Clickable/Cross-Linked
```
FILE: src/pages/transactions.rs
LINE: 169-170
ISSUE: From/To addresses are static text - no navigation to address page
SEVERITY: High
FIX: Wrap addresses in anchor tags that link to explorer page
CODE:
  // Before
  <td style="font-family: monospace; font-size: 11px;">{shorten_hash(&tx.from)}</td>

  // After
  <td style="font-family: monospace; font-size: 11px;">
      <a
          href=format!("/?page=transactions&search={}", tx.from)
          style="color: var(--accent-cyan); text-decoration: none;"
          title={tx.from.clone()}
      >
          {shorten_hash(&tx.from)}
      </a>
  </td>
```

---

### ISSUE H-06: No URL State Preservation
```
FILE: src/pages/transactions.rs
LINE: 7-9
ISSUE: Search input state not synced to URL - refreshing loses search
SEVERITY: High
FIX: Use URL query params for search state with window.history.replaceState
CODE:
  // On search, update URL:
  #[cfg(target_arch = "wasm32")]
  {
      let url = format!("/?page=transactions&q={}", search_address.get());
      web_sys::window()
          .and_then(|w| w.history().ok())
          .map(|h| h.replace_state_with_url(&JsValue::NULL, "", Some(&url)));
  }
```

---

### ISSUE H-07: Header Stats Hidden on Mobile - Data Loss
```
FILE: src/styles.css
LINE: 1136-1138
ISSUE: .header-stats display:none on mobile hides critical TCR/Supply metrics
SEVERITY: High
FIX: Show condensed version or move to footer on mobile
CODE:
  // Before
  @media (max-width: 768px) {
      .header-stats { display: none; }

  // After
  @media (max-width: 768px) {
      .header-stats {
          position: fixed;
          bottom: 40px; /* above footer */
          left: 60px;
          right: 0;
          background: var(--bg-secondary);
          padding: 8px 16px;
          border-top: 1px solid var(--border-color);
          justify-content: space-around;
      }
  }
```

---

### ISSUE H-08: Charts Not Responsive to Container Resize
```
FILE: src/components/charts.rs
LINE: 33-58
ISSUE: SVG viewBox is fixed, no resize observer for dynamic container changes
SEVERITY: High
FIX: Use 100% width with aspect-ratio CSS or add ResizeObserver
CODE:
  // Add CSS to .chart-container:
  .chart-container {
      position: relative;
      width: 100%;
      aspect-ratio: 16 / 9;
  }

  .chart-svg {
      position: absolute;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
  }
```

---

### ISSUE H-09: Network Graph is Just a Table - Missing Visualization
```
FILE: src/pages/network.rs
LINE: 1-109
ISSUE: Page titled "Network Graph" but only shows table - no actual graph visualization
SEVERITY: High
FIX: Implement actual graph visualization using Canvas or SVG force-directed layout
CODE:
  // The .network-canvas CSS exists at styles.css:895-919 but is unused
  // Need to implement actual graph component with nodes/edges
```

---

### ISSUE H-10: Sankey Chart is Just Progress Bars
```
FILE: src/pages/sankey.rs
LINE: 54-71
ISSUE: "Sankey Charts" page shows progress bars, not actual Sankey diagram
SEVERITY: High
FIX: Implement proper Sankey visualization with curved flow paths
```

---

## MEDIUM SEVERITY ISSUES (Polish Issues)

---

### ISSUE M-01: Inconsistent Gap/Spacing in Grid Layouts
```
FILE: src/pages/dashboard.rs
LINE: 38, 83, 148
ISSUE: Inline margin-bottom: 24px hardcoded, should use CSS variable
SEVERITY: Medium
FIX: Add CSS variable and use consistently
CODE:
  // In styles.css :root add:
  --spacing-section: 24px;
  --spacing-card: 20px;

  // Replace inline styles:
  style="margin-bottom: var(--spacing-section);"
```

---

### ISSUE M-02: Duplicate Format Functions Across Files
```
FILE: Multiple files
ISSUE: format_value(), shorten_hash(), format_timestamp() duplicated in 6+ files
SEVERITY: Medium
FIX: Create shared src/format.rs module (already exists but not fully used)
```

---

### ISSUE M-03: Sidebar Toggle Button Missing Focus Ring
```
FILE: src/components/header.rs
LINE: 17-27
ISSUE: Button has no :focus-visible style - keyboard users can't see focus
SEVERITY: Medium
FIX: Add focus style
CODE:
  // In styles.css after line 83:
  .sidebar-toggle:focus-visible {
      outline: 2px solid var(--accent-cyan);
      outline-offset: 2px;
  }
```

---

### ISSUE M-04: Icons Missing aria-label for Screen Readers
```
FILE: src/components/icons.rs
LINE: 1-567
ISSUE: All 50+ icon components have no aria-label or role attributes
SEVERITY: Medium
FIX: Add accessibility props
CODE:
  // Add to each icon component:
  #[component]
  pub fn SearchIcon(#[prop(default = "Search")] aria_label: &'static str) -> impl IntoView {
      view! {
          <svg
              viewBox="0 0 24 24"
              role="img"
              aria-label=aria_label
              ...
          >
```

---

### ISSUE M-05: Table Headers Not Sticky
```
FILE: src/styles.css
LINE: 436-458
ISSUE: Table headers scroll off screen on long tables
SEVERITY: Medium
FIX: Make thead sticky
CODE:
  .data-table thead {
      position: sticky;
      top: 0;
      z-index: 10;
  }

  .data-table th {
      background: var(--bg-tertiary);
  }
```

---

### ISSUE M-06: Error Messages Not User-Friendly
```
FILE: src/pages/collateral.rs
LINE: 56-60
ISSUE: Raw error.to_string() shown to users - may expose internal details
SEVERITY: Medium
FIX: Map errors to user-friendly messages
CODE:
  // Before
  <div class="metric-value red">{err.to_string()}</div>

  // After
  <div class="metric-value red">
      {match err {
          ServerFnError::ServerError(s) if s.contains("timeout") => "Connection timed out. Please try again.",
          ServerFnError::ServerError(s) if s.contains("rate limit") => "Too many requests. Please wait.",
          _ => "Unable to load data. Check your connection.",
      }}
  </div>
```

---

### ISSUE M-07: No Data Staleness Indicator
```
FILE: src/components/footer.rs
LINE: 1-27
ISSUE: Footer shows "Live Data" but no actual timestamp of last update
SEVERITY: Medium
FIX: Add last-updated timestamp to footer
CODE:
  <div class="footer-item">
      <ClockIcon />
      "Updated: "{last_update_signal.get()}
  </div>
```

---

### ISSUE M-08: Gauge Chart Animation Not Smooth
```
FILE: src/components/gauge.rs
LINE: 56-68
ISSUE: Needle rotation uses immediate value, no animation/transition
SEVERITY: Medium
FIX: Add CSS transition to needle group
CODE:
  // Add to styles.css:
  .gauge-needle {
      transition: transform 0.5s ease-out;
  }
```

---

### ISSUE M-09: Card Hover State Too Subtle
```
FILE: src/styles.css
LINE: 316-323
ISSUE: .card has no hover/focus state for interactive cards
SEVERITY: Medium
FIX: Add hover state for clickable cards
CODE:
  .card.clickable:hover {
      border-color: var(--accent-cyan);
      transform: translateY(-2px);
      box-shadow: 0 4px 12px rgba(0, 212, 255, 0.1);
  }
```

---

### ISSUE M-10: Color-Only Status Indicators
```
FILE: src/pages/collateral.rs
LINE: 95-101
ISSUE: Trove status uses color only (red/yellow/green) - inaccessible
SEVERITY: Medium
FIX: Add text labels or icons alongside colors
CODE:
  // Before
  let (status_class, status_text) = if icr < 115.0 {
      ("color: var(--accent-red);", "Critical")

  // After - add icon
  let (status_class, status_icon, status_text) = if icr < 115.0 {
      ("color: var(--accent-red);", "⚠️", "Critical")
```

---

### ISSUE M-11: Buttons Missing Disabled State
```
FILE: src/styles.css
LINE: 567-615
ISSUE: .btn classes have no disabled styling
SEVERITY: Medium
FIX: Add disabled state
CODE:
  .btn:disabled, .btn.disabled {
      opacity: 0.5;
      cursor: not-allowed;
      pointer-events: none;
  }
```

---

### ISSUE M-12: Input Validation Feedback Timing
```
FILE: src/pages/transactions.rs
LINE: 27-36
ISSUE: Validation error appears only after clicking Search, not on blur
SEVERITY: Medium
FIX: Validate on blur for immediate feedback
CODE:
  on:blur=move |_| {
      let addr = search_address.get();
      if !addr.is_empty() && !validate_address(&addr) {
          set_input_error.set(Some("Invalid address format".to_string()));
      }
  }
```

---

## LOW SEVERITY ISSUES (Nice-to-Have)

---

### ISSUE L-01: No Global Search/Command Palette
```
FILE: Global
ISSUE: No way to quickly search across all pages/data
SEVERITY: Low
FIX: Add Cmd+K command palette component
```

---

### ISSUE L-02: No Keyboard Shortcuts
```
FILE: Global
ISSUE: No keyboard navigation shortcuts (R for refresh, etc.)
SEVERITY: Low
FIX: Add keyboard event listener with shortcut hints in tooltip
```

---

### ISSUE L-03: No Onboarding/Walkthrough
```
FILE: Global
ISSUE: New users have no guidance on DeFi terminology or app features
SEVERITY: Low
FIX: Add first-visit tour or contextual tooltips
```

---

### ISSUE L-04: No Comparison Mode
```
FILE: Global
ISSUE: Can't compare time periods or trove performance
SEVERITY: Low
FIX: Add date range selector and comparison view
```

---

### ISSUE L-05: Address Page is Redirect Stub
```
FILE: src/pages/address.rs
LINE: 1-27
ISSUE: Page just redirects to transactions - unnecessary menu item
SEVERITY: Low
FIX: Either remove from sidebar or implement unique functionality
```

---

### ISSUE L-06: Missing Hover Tooltips on Metrics
```
FILE: src/components/metric_card.rs
ISSUE: No tooltip explaining what each metric means
SEVERITY: Low
FIX: Add title attribute or custom tooltip component
CODE:
  <div class="metric-card" title="Total USDFC tokens minted across all Troves">
```

---

### ISSUE L-07: No Copy-to-Clipboard for Addresses/Hashes
```
FILE: src/pages/transactions.rs
LINE: 166
ISSUE: Users can't copy full address/hash from truncated display
SEVERITY: Low
FIX: Add copy button with CopyIcon
CODE:
  <button
      class="btn btn-ghost"
      style="padding: 4px;"
      on:click=move |_| copy_to_clipboard(&tx.hash)
      title="Copy to clipboard"
  >
      <CopyIcon />
  </button>
```

---

### ISSUE L-08: Footer Not Showing Actual Block Number
```
FILE: src/components/footer.rs
LINE: 1-27
ISSUE: Footer shows static "Filecoin Mainnet" but not current block
SEVERITY: Low
FIX: Fetch and display actual block number
```

---

### ISSUE L-09: No Dark/Light Theme Toggle
```
FILE: Global
ISSUE: Only dark theme available, no user preference
SEVERITY: Low
FIX: Add theme toggle with CSS variables swap
```

---

### ISSUE L-10: Chart Tooltips Not Implemented
```
FILE: src/components/charts.rs
ISSUE: AreaChart/BarChart have no hover tooltips showing exact values
SEVERITY: Low
FIX: Add SVG tooltip element that follows mouse
```

---

### ISSUE L-11: Progress Bars Missing Value Labels
```
FILE: src/styles.css
LINE: 709-730
ISSUE: .progress-fill doesn't show percentage value inside
SEVERITY: Low
FIX: Add text inside progress bar when width > 30%
```

---

### ISSUE L-12: No Loading Animation on Refresh Buttons
```
FILE: src/pages/dashboard.rs
LINE: 89-95
ISSUE: Refresh button doesn't indicate loading state
SEVERITY: Low
FIX: Add spinning RefreshIcon or disable during fetch
CODE:
  <button
      class="btn btn-secondary"
      class:loading=move || transactions.loading().get()
      on:click=move |_| transactions.refetch()
      disabled=move || transactions.loading().get()
  >
```

---

## SUMMARY BY SEVERITY

| Severity | Count | Impact |
|----------|-------|--------|
| **Critical** | 3 | Application crashes on malformed data |
| **High** | 10 | Major UX broken (no pagination, no linking, data loss) |
| **Medium** | 12 | Polish issues affecting usability |
| **Low** | 12 | Enhancement opportunities |
| **Total** | **37** | |

---

## PRIORITY FIX ORDER

1. **Week 1 - Critical**: Fix all `.expect()` panics (C-01, C-02, C-03)
2. **Week 2 - Core UX**: Add pagination, sorting, empty states (H-01 to H-04)
3. **Week 3 - Navigation**: Cross-linking, URL state, mobile data (H-05 to H-07)
4. **Week 4 - Visualization**: Fix chart responsiveness, implement actual graphs (H-08 to H-10)
5. **Ongoing**: Medium and low issues as time permits
