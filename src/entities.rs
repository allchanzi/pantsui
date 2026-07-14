#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Version,
    Mode,
    FileExplorer,
    Main,
    Runs,
}

impl Panel {
    pub fn shortcut(self) -> &'static str {
        match self {
            Self::Version => "1",
            Self::Mode => "2",
            Self::FileExplorer => "3",
            Self::Main => "4",
            Self::Runs => "5",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Version => "Version",
            Self::Mode => "Mode",
            Self::FileExplorer => "Source Tree",
            Self::Main => "Active Area",
            Self::Runs => "Runs",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Version => Self::Mode,
            Self::Mode => Self::FileExplorer,
            Self::FileExplorer => Self::Main,
            Self::Main => Self::Runs,
            Self::Runs => Self::Version,
        }
    }

    pub fn move_left(self) -> Self {
        match self {
            Self::Main => Self::Version,
            Self::Runs => Self::FileExplorer,
            current => current,
        }
    }

    pub fn move_right(self) -> Self {
        match self {
            Self::Version => Self::Main,
            Self::FileExplorer => Self::Runs,
            current => current,
        }
    }

    pub fn move_up(self) -> Self {
        match self {
            Self::Mode => Self::Version,
            Self::FileExplorer => Self::Mode,
            Self::Runs => Self::Main,
            current => current,
        }
    }

    pub fn move_down(self) -> Self {
        match self {
            Self::Version => Self::Mode,
            Self::Mode => Self::FileExplorer,
            Self::Main => Self::Runs,
            current => current,
        }
    }
}
