use leptos::*;

#[component]
pub fn GaugeChart(
    value: f64,
    #[prop(default = 0.0)] min: f64,
    #[prop(default = 100.0)] max: f64,
    #[prop(default = "TCR")] label: &'static str,
    #[prop(default = "%")] suffix: &'static str,
) -> impl IntoView {
    let normalized = ((value - min) / (max - min)).clamp(0.0, 1.0);
    let angle = -90.0 + normalized * 180.0;
    
    let get_color = move || {
        if value < 120.0 { "var(--accent-red)" }
        else if value < 150.0 { "var(--accent-yellow)" }
        else { "var(--accent-cyan)" }
    };
    
    view! {
        <div class="gauge-container">
            <svg class="gauge-svg" viewBox="0 0 200 120">
                // Background arc
                <path
                    d="M 20 100 A 80 80 0 0 1 180 100"
                    fill="none"
                    stroke="var(--bg-tertiary)"
                    stroke-width="16"
                    stroke-linecap="round"
                />
                // Colored zones
                <path
                    d="M 20 100 A 80 80 0 0 1 60 35"
                    fill="none"
                    stroke="var(--accent-red)"
                    stroke-width="16"
                    stroke-linecap="round"
                    opacity="0.3"
                />
                <path
                    d="M 60 35 A 80 80 0 0 1 100 20"
                    fill="none"
                    stroke="var(--accent-yellow)"
                    stroke-width="16"
                    stroke-linecap="round"
                    opacity="0.3"
                />
                <path
                    d="M 100 20 A 80 80 0 0 1 180 100"
                    fill="none"
                    stroke="var(--accent-cyan)"
                    stroke-width="16"
                    stroke-linecap="round"
                    opacity="0.3"
                />
                // Needle
                <g transform=format!("rotate({} 100 100)", angle)>
                    <line 
                        x1="100" 
                        y1="100" 
                        x2="100" 
                        y2="35" 
                        stroke={get_color()} 
                        stroke-width="3"
                        stroke-linecap="round"
                    />
                    <circle cx="100" cy="100" r="8" fill={get_color()} />
                </g>
                // Scale labels
                <text x="20" y="115" fill="var(--text-muted)" font-size="10" text-anchor="middle">"110%"</text>
                <text x="100" y="12" fill="var(--text-muted)" font-size="10" text-anchor="middle">"150%"</text>
                <text x="180" y="115" fill="var(--text-muted)" font-size="10" text-anchor="middle">"200%+"</text>
            </svg>
            <div class="gauge-value" style=format!("color: {}", get_color())>
                {format!("{:.1}{}", value, suffix)}
            </div>
            <div class="gauge-label">{label}</div>
        </div>
    }
}
