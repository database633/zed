use std::cmp::Ordering;

use gpui::Entity;
use ui::prelude::*;
use ui::{Tab, TabPosition, Tooltip};

use crate::{MultiWorkspace, RemovalIntent, Workspace};

thread_local! {
    /// Last number of project tabs rendered. Used to log strip changes once
    /// instead of on every frame.
    static LAST_PROJECT_TAB_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(usize::MAX) };
}

/// The text shown on a project tab: the file names of the workspace's root
/// paths, joined with a middle dot. Empty projects (no worktrees) show
/// `Untitled`.
fn workspace_tab_label(workspace: &Entity<Workspace>, cx: &App) -> SharedString {
    let (root_paths, path_style) = {
        let workspace = workspace.read(cx);
        let path_style = workspace.project().read(cx).path_style(cx);
        (workspace.root_paths(cx), path_style)
    };

    let names: Vec<String> = root_paths
        .iter()
        .map(|root_path| {
            let root_path = root_path.as_ref();
            path_style
                .file_name(root_path)
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| root_path.to_string_lossy().into_owned())
        })
        .collect();

    if names.is_empty() {
        "Untitled".into()
    } else {
        names.join(" · ").into()
    }
}

/// Renders the browser-style project tabs as a single horizontal row, one tab
/// per workspace held by this window. Clicking a tab activates that workspace;
/// the close button removes it. `end_control` is the trailing "add project"
/// button, provided by the title bar (which owns the recent-projects popover).
///
/// The row is laid out by hand rather than with `ui::TabBar`. `TabBar` renders
/// a full-width element that does not grow, and a `width: 100%` child of a
/// flex-grow-sized parent resolves to zero width, so a `TabBar` placed in a
/// flex row collapses and nothing is visible.
pub fn render_project_tabs(
    multi_workspace: &MultiWorkspace,
    _window: &mut Window,
    cx: &mut Context<MultiWorkspace>,
    end_control: Option<AnyElement>,
) -> AnyElement {
    let active_workspace = multi_workspace.workspace().clone();
    let workspaces: Vec<_> = multi_workspace.workspaces().cloned().collect();
    let last_index = workspaces.len().saturating_sub(1);

    LAST_PROJECT_TAB_COUNT.with(|last| {
        if last.get() != workspaces.len() {
            last.set(workspaces.len());
            log::info!("project tabs: rendering {} tab(s)", workspaces.len());
        }
    });

    // Content-sized on purpose: a `flex_1` scroll container consumes all the
    // remaining width and pushes the trailing "add project" control out of the
    // clipped area, so it never renders.
    let mut tabs = h_flex()
        .id("project-tabs-list")
        .flex_none()
        .h(Tab::container_height(cx));

    for (ix, workspace) in workspaces.iter().enumerate() {
        let is_active = workspace == &active_workspace;
        let group_name = SharedString::from(format!("project-tab-{ix}"));
        let label = workspace_tab_label(workspace, cx);

        let workspace_to_activate = workspace.clone();
        let workspace_to_close = workspace.clone();

        let position = if ix == 0 {
            TabPosition::First
        } else if ix == last_index {
            TabPosition::Last
        } else {
            TabPosition::Middle(Ordering::Equal)
        };

        let tab = Tab::new(("project-tab", ix))
            .position(position)
            .group(group_name.clone())
            .toggle_state(is_active)
            .on_click(cx.listener(move |multi_workspace, _event, window, cx| {
                log::info!("project tabs: activating workspace tab {ix}");
                multi_workspace.activate(workspace_to_activate.clone(), None, window, cx);
            }))
            .end_slot(
                IconButton::new(("close-project-tab", ix), IconName::Close)
                    .icon_size(IconSize::XSmall)
                    .icon_color(Color::Muted)
                    .visible_on_hover(group_name)
                    .tooltip(Tooltip::text("Close Project"))
                    .on_click(cx.listener(move |multi_workspace, _event, window, cx| {
                        log::info!("project tabs: closing workspace tab {ix}");
                        multi_workspace
                            .remove(
                                [workspace_to_close.clone()],
                                RemovalIntent::CloseProject,
                                window,
                                cx,
                            )
                            .detach_and_log_err(cx);
                    })),
            )
            .child(
                h_flex()
                    .gap_1()
                    .child(
                        Icon::new(IconName::Folder)
                            .size(IconSize::XSmall)
                            .color(Color::Muted),
                    )
                    .child(Label::new(label).truncate()),
            );

        tabs = tabs.child(tab);
    }

    let mut row = h_flex()
        .id("project-tabs")
        .flex_none()
        .w_full()
        .h(Tab::container_height(cx))
        .bg(cx.theme().colors().tab_bar_background)
        .border_b_1()
        .border_color(cx.theme().colors().border)
        .child(tabs);

    if let Some(end_control) = end_control {
        row = row.child(end_control);
    }

    row.into_any_element()
}
