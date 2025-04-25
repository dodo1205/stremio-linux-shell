pub const BOOL_PROPERTIES: &[&str] = &[
    "pause",
    "mute",
    "buffering",
    "osc",
    "input-default-bindings",
    "input-vo-keyboard",
];

pub const STRING_PROPERTIES: &[&str] = &[
    "path",
    "mpv-version",
    "ffmpeg-version",
    "hwdec",
    "vo",
    "track-list",
    "sub-color",
    "sub-back-color",
    "sub-border-color",
];

pub const FLOAT_PROPERTIES: &[&str] = &[
    "time-pos",
    "duration",
    "volume",
    "speed",
    "sub-pos",
    "sub-scale",
    "sub-delay",
];

pub const INT_PROPERTIES: &[&str] = &["sid", "aid"];
