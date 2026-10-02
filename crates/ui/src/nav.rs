//! The navigation inventory: every reference page of the shell.
//!
//! One entry per reference page identified in the current inventory
//! (docs/SPEC-AMENDMENTS.md, A23): the navigation entry exists even while
//! the page itself is not built, so the route opens and honestly says
//! "Not built yet" instead of hiding the route or faking functionality.

/// One navigation entry: where it links and what it is called.
pub struct NavItem {
    /// The route path, starting from the root.
    pub path: &'static str,
    /// The visible label in the sidebar.
    pub label: &'static str,
}

/// Every reference page the persistent shell navigates to.
pub const NAV_ITEMS: &[NavItem] = &[
    NavItem {
        path: "/",
        label: "Dashboard",
    },
    NavItem {
        path: "/inbox",
        label: "Inbox",
    },
    NavItem {
        path: "/notifications",
        label: "Notifications",
    },
    NavItem {
        path: "/search",
        label: "Search",
    },
    NavItem {
        path: "/customers",
        label: "Customers",
    },
    NavItem {
        path: "/organizations",
        label: "Organizations",
    },
    NavItem {
        path: "/ai",
        label: "AI Center",
    },
    NavItem {
        path: "/issues",
        label: "Issues",
    },
    NavItem {
        path: "/incidents",
        label: "Incidents",
    },
    NavItem {
        path: "/knowledge",
        label: "Knowledge",
    },
    NavItem {
        path: "/docs",
        label: "Docs",
    },
    NavItem {
        path: "/custom-objects",
        label: "Objects",
    },
    NavItem {
        path: "/connectors",
        label: "Connectors",
    },
    NavItem {
        path: "/graph",
        label: "Graph",
    },
    NavItem {
        path: "/operations",
        label: "Operations",
    },
    NavItem {
        path: "/reports",
        label: "Reports",
    },
    NavItem {
        path: "/outreach",
        label: "Outreach",
    },
    NavItem {
        path: "/automation",
        label: "Automation",
    },
    NavItem {
        path: "/sync-health",
        label: "Sync Health",
    },
    NavItem {
        path: "/settings",
        label: "Settings",
    },
];

#[cfg(test)]
mod tests {
    use super::NAV_ITEMS;
    use std::collections::HashSet;

    #[test]
    fn every_reference_page_is_navigable() {
        assert_eq!(NAV_ITEMS.len(), 20, "the reference inventory has 20 pages");
    }

    #[test]
    fn paths_and_labels_are_unique() {
        let paths: HashSet<&str> = NAV_ITEMS.iter().map(|item| item.path).collect();
        assert_eq!(paths.len(), NAV_ITEMS.len(), "paths must be unique");
        let labels: HashSet<&str> = NAV_ITEMS.iter().map(|item| item.label).collect();
        assert_eq!(labels.len(), NAV_ITEMS.len(), "labels must be unique");
    }
}
