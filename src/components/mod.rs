pub mod header;
pub mod sidebar;
pub mod footer;
pub mod icons;
pub mod charts;
pub mod gauge;
pub mod metric_card;
pub mod data_table;
pub mod error_boundary;
pub mod loading;
pub mod memo;
pub mod tabs;
pub mod controls;
pub mod advanced_chart;
pub mod pagination;

// Re-export commonly used loading/skeleton components for convenience
pub use loading::{
    Skeleton, CardSkeleton, MetricCardSkeleton, MetricRowSkeleton,
    ActivityItemSkeleton, TableRowSkeleton, TableSkeleton, ChartSkeleton,
    LoadingSpinner, InlineSpinner, LoadingOverlay, EmptyState, LiveIndicator,
    ProgressBar,
};

// Re-export pagination components
pub use pagination::{Pagination, PaginationWithSize};
