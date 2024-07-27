#![allow(clippy::manual_is_ascii_check)]

use scangen::{DfaData, FindMatches, Scanner, ScannerBuilder, ScannerModeData};

const DFAS: &[DfaData] = &[
    /* 0 */
    (
        "\\r\\n|\\r|\\n",
        &[1, 2],
        &[(0, 2), (0, 0), (2, 3)],
        &[(0, (0, 2)), (0, (1, 1)), (2, (1, 1))],
    ),
    /* 1 */
    (
        "[\\s--\\r\\n]+",
        &[1],
        &[(0, 1), (1, 2)],
        &[(0, (2, 1)), (1, (2, 1))],
    ),
    /* 2 */
    (
        "(//.*(\\r\\n|\\r|\\n))",
        &[3, 4],
        &[(0, 1), (1, 2), (2, 5), (0, 0), (5, 6)],
        &[
            (0, (3, 1)),
            (1, (3, 2)),
            (2, (4, 2)),
            (2, (0, 4)),
            (2, (1, 3)),
            (4, (1, 3)),
        ],
    ),
    /* 3 */
    (
        "(/\\*.*?\\*/)",
        &[4],
        &[(0, 1), (1, 2), (2, 3), (3, 5), (0, 0)],
        &[
            (0, (3, 2)),
            (1, (3, 4)),
            (2, (5, 3)),
            (3, (5, 1)),
            (3, (4, 3)),
        ],
    ),
    /* 4 */
    (
        "%start",
        &[6],
        &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6), (0, 0)],
        &[
            (0, (6, 1)),
            (1, (7, 2)),
            (2, (8, 4)),
            (3, (8, 6)),
            (4, (9, 5)),
            (5, (10, 3)),
        ],
    ),
    /* 5 */
    (
        "%title",
        &[6],
        &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6), (0, 0)],
        &[
            (0, (6, 1)),
            (1, (8, 3)),
            (2, (8, 4)),
            (3, (11, 2)),
            (4, (12, 5)),
            (5, (13, 6)),
        ],
    ),
    /* 6 */
    (
        "%comment",
        &[8],
        &[
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 8),
            (0, 0),
        ],
        &[
            (0, (6, 1)),
            (1, (14, 2)),
            (2, (15, 3)),
            (3, (16, 4)),
            (4, (16, 5)),
            (5, (13, 6)),
            (6, (17, 7)),
            (7, (8, 8)),
        ],
    ),
    /* 7 */
    (
        "%user_type",
        &[10],
        &[
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 8),
            (8, 9),
            (9, 10),
            (0, 0),
        ],
        &[
            (0, (6, 1)),
            (1, (18, 2)),
            (2, (7, 3)),
            (3, (13, 5)),
            (4, (13, 10)),
            (5, (10, 6)),
            (6, (19, 7)),
            (7, (8, 8)),
            (8, (20, 9)),
            (9, (21, 4)),
        ],
    ),
    /* 8 */
    ("=", &[1], &[(0, 1), (0, 0)], &[(0, (22, 1))]),
    /* 9 */
    (
        "%grammar_type",
        &[13],
        &[
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 8),
            (8, 9),
            (9, 10),
            (10, 11),
            (11, 12),
            (12, 13),
            (0, 0),
        ],
        &[
            (0, (6, 1)),
            (1, (23, 2)),
            (2, (10, 5)),
            (3, (10, 8)),
            (4, (9, 3)),
            (5, (9, 7)),
            (6, (16, 4)),
            (7, (16, 6)),
            (8, (19, 9)),
            (9, (8, 10)),
            (10, (20, 11)),
            (11, (21, 12)),
            (12, (13, 13)),
        ],
    ),
    /* 10 */
    (
        "%line_comment",
        &[13],
        &[
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 8),
            (8, 9),
            (9, 10),
            (10, 11),
            (11, 12),
            (12, 13),
            (0, 0),
        ],
        &[
            (0, (6, 1)),
            (1, (12, 2)),
            (2, (11, 3)),
            (3, (17, 6)),
            (4, (17, 12)),
            (5, (13, 4)),
            (6, (13, 7)),
            (7, (19, 8)),
            (8, (14, 9)),
            (9, (15, 11)),
            (10, (16, 5)),
            (11, (16, 10)),
            (12, (8, 13)),
        ],
    ),
    /* 11 */
    (
        "%block_comment",
        &[14],
        &[
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 8),
            (8, 9),
            (9, 10),
            (10, 11),
            (11, 12),
            (12, 13),
            (13, 14),
            (0, 0),
        ],
        &[
            (0, (6, 1)),
            (1, (24, 2)),
            (2, (12, 3)),
            (3, (15, 6)),
            (4, (15, 9)),
            (5, (14, 4)),
            (6, (14, 7)),
            (7, (25, 8)),
            (8, (19, 5)),
            (9, (16, 10)),
            (10, (16, 11)),
            (11, (13, 12)),
            (12, (17, 13)),
            (13, (8, 14)),
        ],
    ),
    /* 12 */
    (
        "%auto_newline_off",
        &[17],
        &[
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 8),
            (8, 9),
            (9, 10),
            (10, 11),
            (11, 12),
            (12, 13),
            (13, 14),
            (14, 15),
            (15, 16),
            (16, 17),
            (0, 0),
        ],
        &[
            (0, (6, 1)),
            (1, (9, 2)),
            (2, (18, 3)),
            (3, (8, 4)),
            (4, (15, 7)),
            (5, (15, 15)),
            (6, (19, 5)),
            (7, (19, 9)),
            (8, (17, 10)),
            (9, (17, 11)),
            (10, (13, 6)),
            (11, (13, 12)),
            (12, (26, 13)),
            (13, (12, 14)),
            (14, (11, 8)),
            (15, (27, 16)),
            (16, (27, 17)),
        ],
    ),
    /* 13 */
    (
        "%auto_ws_off",
        &[12],
        &[
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 8),
            (8, 9),
            (9, 10),
            (10, 11),
            (11, 12),
            (0, 0),
        ],
        &[
            (0, (6, 1)),
            (1, (9, 2)),
            (2, (18, 3)),
            (3, (8, 4)),
            (4, (15, 7)),
            (5, (15, 10)),
            (6, (19, 5)),
            (7, (19, 8)),
            (8, (26, 9)),
            (9, (7, 6)),
            (10, (27, 11)),
            (11, (27, 12)),
        ],
    ),
    /* 14 */
    (
        "%on",
        &[3],
        &[(0, 1), (1, 2), (2, 3), (0, 0)],
        &[(0, (6, 1)), (1, (15, 2)), (2, (17, 3))],
    ),
    /* 15 */
    (
        "%enter",
        &[6],
        &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6), (0, 0)],
        &[
            (0, (6, 1)),
            (1, (13, 3)),
            (2, (13, 5)),
            (3, (17, 4)),
            (4, (8, 2)),
            (5, (10, 6)),
        ],
    ),
    /* 16 */
    (
        "%%",
        &[2],
        &[(0, 1), (1, 2), (0, 0)],
        &[(0, (6, 1)), (1, (6, 2))],
    ),
    /* 17 */
    (
        "::",
        &[2],
        &[(0, 1), (1, 2), (0, 0)],
        &[(0, (28, 1)), (1, (28, 2))],
    ),
    /* 18 */
    (":", &[1], &[(0, 1), (0, 0)], &[(0, (28, 1))]),
    /* 19 */
    (";", &[1], &[(0, 1), (0, 0)], &[(0, (29, 1))]),
    /* 20 */
    ("\\|", &[1], &[(0, 1), (0, 0)], &[(0, (30, 1))]),
    /* 21 */
    ("<", &[1], &[(0, 1), (0, 0)], &[(0, (31, 1))]),
    /* 22 */
    (">", &[1], &[(0, 1), (0, 0)], &[(0, (32, 1))]),
    /* 23 */
    (
        "\"(\\\\.|[^\\\\])*?\"",
        &[2],
        &[(0, 1), (1, 4), (0, 0), (4, 7), (7, 8)],
        &[
            (0, (33, 1)),
            (1, (33, 2)),
            (1, (34, 4)),
            (1, (35, 3)),
            (3, (33, 2)),
            (3, (34, 4)),
            (3, (35, 3)),
            (4, (4, 3)),
        ],
    ),
    /* 24 */
    (
        "\'(\\\\\'|[^\'])*?\'",
        &[2],
        &[(0, 1), (1, 4), (0, 0), (4, 7), (7, 8)],
        &[
            (0, (36, 1)),
            (1, (36, 2)),
            (1, (34, 4)),
            (1, (37, 3)),
            (3, (36, 2)),
            (3, (34, 4)),
            (3, (37, 3)),
            (4, (36, 3)),
        ],
    ),
    /* 25 */
    (
        "\\u{2F}(\\\\.|[^\\\\])*?\\u{2F}",
        &[2],
        &[(0, 1), (1, 4), (0, 0), (4, 7), (7, 8)],
        &[
            (0, (38, 1)),
            (1, (38, 2)),
            (1, (34, 4)),
            (1, (35, 3)),
            (3, (38, 2)),
            (3, (34, 4)),
            (3, (35, 3)),
            (4, (4, 3)),
        ],
    ),
    /* 26 */
    ("\\(", &[1], &[(0, 1), (0, 0)], &[(0, (39, 1))]),
    /* 27 */
    ("\\)", &[1], &[(0, 1), (0, 0)], &[(0, (40, 1))]),
    /* 28 */
    ("\\[", &[1], &[(0, 1), (0, 0)], &[(0, (41, 1))]),
    /* 29 */
    ("\\]", &[1], &[(0, 1), (0, 0)], &[(0, (42, 1))]),
    /* 30 */
    ("\\{", &[1], &[(0, 1), (0, 0)], &[(0, (43, 1))]),
    /* 31 */
    ("\\}", &[1], &[(0, 1), (0, 0)], &[(0, (44, 1))]),
    /* 32 */
    (
        "[a-zA-Z_][a-zA-Z0-9_]*",
        &[1],
        &[(0, 1), (1, 2)],
        &[(0, (45, 1)), (1, (46, 1))],
    ),
    /* 33 */
    (
        "%scanner",
        &[8],
        &[
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 8),
            (0, 0),
        ],
        &[
            (0, (6, 1)),
            (1, (7, 2)),
            (2, (14, 3)),
            (3, (9, 4)),
            (4, (17, 5)),
            (5, (17, 6)),
            (6, (13, 7)),
            (7, (10, 8)),
        ],
    ),
    /* 34 */
    (",", &[1], &[(0, 1), (0, 0)], &[(0, (47, 1))]),
    /* 35 */
    (
        "%sc",
        &[3],
        &[(0, 1), (1, 2), (2, 3), (0, 0)],
        &[(0, (6, 1)), (1, (7, 2)), (2, (14, 3))],
    ),
    /* 36 */
    (
        "%push",
        &[5],
        &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (0, 0)],
        &[
            (0, (6, 1)),
            (1, (21, 2)),
            (2, (18, 3)),
            (3, (7, 4)),
            (4, (48, 5)),
        ],
    ),
    /* 37 */
    (
        "%pop",
        &[4],
        &[(0, 1), (1, 2), (2, 3), (3, 4), (0, 0)],
        &[(0, (6, 1)), (1, (21, 3)), (2, (21, 4)), (3, (15, 2))],
    ),
    /* 38 */
    ("\\^", &[1], &[(0, 1), (0, 0)], &[(0, (49, 1))]),
    /* 39 */
    (".", &[1], &[(0, 1), (0, 0)], &[(0, (4, 1))]),
];

const MODES: &[ScannerModeData] = &[];

fn matches_char_class(c: char, char_class: usize) -> bool {
    match char_class {
        /* \r */
        0 => c == '\r',
        /* \n */
        1 => c == '\n',
        /* [\s--\r\n] */
        2 => c.is_whitespace() && !(c == '\r' || c == '\n'),
        /* / */
        3 => c == '/',
        /* . */
        4 => c != '\n' && c != '\r',
        /* \* */
        5 => c == '*',
        /* % */
        6 => c == '%',
        /* s */
        7 => c == 's',
        /* t */
        8 => c == 't',
        /* a */
        9 => c == 'a',
        /* r */
        10 => c == 'r',
        /* i */
        11 => c == 'i',
        /* l */
        12 => c == 'l',
        /* e */
        13 => c == 'e',
        /* c */
        14 => c == 'c',
        /* o */
        15 => c == 'o',
        /* m */
        16 => c == 'm',
        /* n */
        17 => c == 'n',
        /* u */
        18 => c == 'u',
        /* _ */
        19 => c == '_',
        /* y */
        20 => c == 'y',
        /* p */
        21 => c == 'p',
        /* = */
        22 => c == '=',
        /* g */
        23 => c == 'g',
        /* b */
        24 => c == 'b',
        /* k */
        25 => c == 'k',
        /* w */
        26 => c == 'w',
        /* f */
        27 => c == 'f',
        /* : */
        28 => c == ':',
        /* ; */
        29 => c == ';',
        /* \| */
        30 => c == '|',
        /* < */
        31 => c == '<',
        /* > */
        32 => c == '>',
        /* " */
        33 => c == '\"',
        /* \\ */
        34 => c == '\\',
        /* [^\\] */
        35 => c != '\\',
        /* ' */
        36 => c == '\'',
        /* [^'] */
        37 => c != '\'',
        /* \u{2F} */
        38 => c == '/',
        /* \( */
        39 => c == '(',
        /* \) */
        40 => c == ')',
        /* \[ */
        41 => c == '[',
        /* \] */
        42 => c == ']',
        /* \{ */
        43 => c == '{',
        /* \} */
        44 => c == '}',
        /* [a-zA-Z_] */
        45 => ('a'..='z').contains(&c) || ('A'..='Z').contains(&c) || c == '_',
        /* [a-zA-Z0-9_] */
        46 => {
            ('a'..='z').contains(&c)
                || ('A'..='Z').contains(&c)
                || ('0'..='9').contains(&c)
                || c == '_'
        }
        /* , */
        47 => c == ',',
        /* h */
        48 => c == 'h',
        /* \^ */
        49 => c == '^',
        _ => false,
    }
}

pub(crate) fn create_scanner() -> Scanner {
    ScannerBuilder::new()
        .add_dfa_data(DFAS)
        .add_scanner_mode_data(MODES)
        .build()
}

pub(crate) fn create_find_iter<'r, 'h>(
    scanner: &'r mut Scanner,
    input: &'h str,
) -> FindMatches<'r, 'h> {
    scanner.find_iter(input, matches_char_class)
}
