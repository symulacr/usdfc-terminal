use leptos::*;
use crate::components::icons::SearchIcon;

#[component]
pub fn DataTable<T: Clone + 'static>(
    headers: Vec<&'static str>,
    data: Vec<T>,
    render_row: fn(&T) -> View,
    #[prop(default = "No data available")] empty_title: &'static str,
    #[prop(default = "Try adjusting your filters or check back later.")] empty_desc: &'static str,
) -> impl IntoView {
    let col_count = headers.len();

    view! {
        <div class="table-container">
            <table class="data-table">
                <thead>
                    <tr>
                        {headers.iter().map(|h| view! { <th>{*h}</th> }).collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {if data.is_empty() {
                        view! {
                            <tr>
                                <td colspan=col_count.to_string() style="padding: 0;">
                                    <div class="empty-state">
                                        <div class="empty-state-icon"><SearchIcon /></div>
                                        <div class="empty-state-title">{empty_title}</div>
                                        <div class="empty-state-desc">{empty_desc}</div>
                                    </div>
                                </td>
                            </tr>
                        }.into_view()
                    } else {
                        data.iter().map(render_row).collect_view()
                    }}
                </tbody>
            </table>
        </div>
    }
}
