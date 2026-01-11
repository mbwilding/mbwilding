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
    text: "#1f2328",
    icon: "#59636e",
    star: "#e3b341",
};

pub const DARK: Theme = Theme {
    name: "dark",
    bg: "#0d1117",
    text: "#f0f6fc",
    icon: "#7d8590",
    star: "#e3b341",
};

pub const ALL: [Theme; 2] = [LIGHT, DARK];
