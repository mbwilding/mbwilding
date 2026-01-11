#[derive(Clone, Copy)]
pub struct Theme {
    pub name: &'static str,
    pub bg: &'static str,
    pub text: &'static str,
    pub icon: &'static str,
    pub star: &'static str,
}

pub const LIGHT: Theme = Theme {
    name: "light",
    bg: "#ffffff",
    text: "#333333",
    icon: "#586069",
    star: "#e3b341",
};

pub const DARK: Theme = Theme {
    name: "dark",
    bg: "#0d1117",
    text: "#c9d1d9",
    icon: "#8b949e",
    star: "#e3b341",
};

pub const ALL: [Theme; 2] = [LIGHT, DARK];
