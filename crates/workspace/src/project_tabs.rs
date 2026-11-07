use gpui::Entity;
use ui::prelude::*;
use ui::{Tab, TabBar, Tooltip};

use crate::{MultiWorkspace, RemovalIntent, Workspace};

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

/// Renders a browser-style horizontal strip of tabs, one per workspace held by
/// this window. Clicking a tab activates that workspace; the close button
/// removes it. The "add project" control lives in the title bar, which owns the
/// recent-projects popover.
pub fn render_project_tabs(
    multi_workspace: &MultiWorkspace,
    window: &mut Window,
    cx: &mut Context<MultiWorkspace>,
) -> AnyElement {
    let active_workspace = multi_workspace.workspace().clone();

    let mut tab_bar = TabBar::new("project-tabs-bar");

    for (ix, workspace) in multi_workspace.workspaces().enumerate() {
        let is_active = workspace == &active_workspace;
        let group_name = SharedString::from(format!("project-tab-{ix}"));
        let label = workspace_tab_label(workspace, cx);

        let workspace_to_activate = workspace.clone();
        let workspace_to_close = workspace.clone();

        let tab = Tab::new(("project-tab", ix))
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

        tab_bar = tab_bar.child(tab);
    }

    tab_bar.into_any_element()
}
