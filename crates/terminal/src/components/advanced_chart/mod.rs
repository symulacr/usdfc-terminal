//! Advanced Chart Component Module
//!
//! TradingView-style chart container with multiple series, resolution/lookback controls,
//! wallet analytics, and interactive features.

mod container;
mod header;
mod canvas;
mod series;
mod tooltip;
mod legend;

pub use container::AdvancedChart;
pub use header::ChartHeader;
pub use canvas::ChartCanvas;
pub use series::{AreaSeries, LineSeries, CandlestickSeries, BarSeries};
pub use tooltip::ChartTooltip;
pub use legend::ChartLegend;
