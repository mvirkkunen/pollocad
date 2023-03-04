enum class MouseFlags {
    Left = (1 << 1),
    Middle = (1 << 2),
    Right = (1 << 3),
    Shift = (1 << 4),
    Ctrl = (1 << 5),
    Alt = (1 << 6),
    ButtonChange = (1 << 7),
};

enum class BooleanOp {
    Union = 1,
    Difference = 2,
    Intersection = 3,
};
