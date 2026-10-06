//! Geometry of the map of a training path: where each course sits and how prerequisite links are drawn.
//!
//! The map is HTML (an ordered list of course cards, which works without styles or scripts) with the links
//! drawn in SVG behind it. For the two to line up without any script, every dimension is fixed here and
//! handed to the style sheet as custom properties (see [`Layout::style`] and `static/css/paths.css`).
//! Lengths are in **units of 1/16 rem**, so the whole map follows the reader's font size.
//!
//! Two layouts are computed for the same cards, and the style sheet shows one:
//!
//! - **wide**: one column per stage, courses stacked in each column. Columns share the available width, so
//!   horizontal positions are fractions of it: the SVG is stretched horizontally (`COLUMN` units per column,
//!   `preserveAspectRatio="none"`) while its height is exact. Cards leave a tenth of their column free on each
//!   side; links travel in those corridors and in the gaps between rows, never behind a card;
//! - **narrow** (phones): every course on its own row, in order, with a rail on the left where each link is
//!   a bracket in its own lane, like the graph of `git log`.
//!
//! Links always go forwards (down the list, or to a column further right): the catalogue compiler refuses a
//! path that lists a course before one of its prerequisites.

use std::fmt::Write;

/// Width of a column in the wide SVG. Arbitrary: the SVG is stretched to the real width.
const COLUMN: i32 = 1000;
/// Part of a column left free on each side of a card. `paths.css` uses the same tenth.
const INSET: i32 = COLUMN / 10;
/// Height of a stage title above its column.
const WIDE_HEAD: i32 = 48;
const WIDE_NODE: i32 = 136;
const WIDE_GAP: i32 = 40;
/// Free space above the first row: a link can pass there.
const WIDE_TOP: i32 = 24;
const WIDE_BOTTOM: i32 = 8;
/// Where a link that goes around its own column leaves a card, below the middle of its side.
const WIDE_SIDE_EXIT: i32 = 30;

const NARROW_STAGE: i32 = 44;
const NARROW_NODE: i32 = 128;
const NARROW_GAP: i32 = 14;
/// Distance between two lanes of the rail.
const LANE: i32 = 12;
/// Distance between the cards and the first lane.
const LANE_START: i32 = 16;
/// Free space left of the last lane.
const RAIL_MARGIN: i32 = 6;
/// A link leaves a card this far below the middle of its side, and arrives this far above.
const NARROW_PORT: i32 = 14;
const CORNER: i32 = 6;
const ARROW_LENGTH: i32 = 7;
const ARROW_HALF: i32 = 4;

/// Where a course sits.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Slot {
    pub column: usize,
    pub row: usize,
    /// In the wide layout, a link arrives on the left side of the card.
    pub enters_left: bool,
    /// In the wide layout, a link arrives on the top side of the card.
    pub enters_top: bool,
}

/// A prerequisite link, from the course to complete first to the course it opens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub from: usize,
    pub to: usize,
    /// SVG path in the wide layout.
    pub wide: String,
    /// SVG path in the narrow layout.
    pub narrow: String,
    /// Points of the arrow head in the narrow layout. The wide one is stretched, so its arrow heads are
    /// drawn by the style sheet on the cards (see [`Slot::enters_left`]).
    pub narrow_arrow: String,
}

/// The map of a path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    pub columns: usize,
    /// One slot per course, in path order.
    pub slots: Vec<Slot>,
    pub links: Vec<Link>,
    /// Stage titles are shown.
    titled: bool,
    rows: usize,
    lanes: usize,
    narrow_height: i32,
}

/// A length in rem, as short as it can be written.
fn rem(units: i32) -> String {
    let text = format!("{:.4}", f64::from(units) / 16.0);
    format!("{}rem", text.trim_end_matches('0').trim_end_matches('.'))
}

fn wide_top(row: usize) -> i32 {
    WIDE_TOP + row as i32 * (WIDE_NODE + WIDE_GAP)
}

fn wide_middle(row: usize) -> i32 {
    wide_top(row) + WIDE_NODE / 2
}

/// Height of the free corridor above `row`.
fn wide_corridor(row: usize) -> i32 {
    if row == 0 {
        WIDE_TOP / 2
    } else {
        wide_top(row) - WIDE_GAP / 2
    }
}

/// The wide path of a link between two slots.
fn wide_path(from: Slot, to: Slot) -> String {
    let left = |column: usize| column as i32 * COLUMN + INSET;
    let right = |column: usize| (column as i32 + 1) * COLUMN - INSET;
    let (y0, y1) = (wide_middle(from.row), wide_middle(to.row));
    if from.column == to.column {
        if to.row == from.row + 1 {
            // The card just below: straight down.
            let x = from.column as i32 * COLUMN + COLUMN / 2;
            return format!("M{x} {} V{}", wide_top(from.row) + WIDE_NODE, wide_top(to.row));
        }
        // Further down the same column: around the cards in between, through the corridor on their left.
        let (x, out) = (left(from.column), from.column as i32 * COLUMN + INSET / 3);
        let y0 = y0 + WIDE_SIDE_EXIT;
        return format!("M{x} {y0} Q{out} {y0} {out} {} V{} Q{out} {y1} {x} {y1}", y0 + 16, y1 - 16);
    }
    let (x0, x1) = (right(from.column), left(to.column));
    if to.column == from.column + 1 {
        if y0 == y1 {
            return format!("M{x0} {y0} H{x1}");
        }
        // An S in the corridor between the two columns.
        let middle = to.column as i32 * COLUMN;
        return format!("M{x0} {y0} C{middle} {y0} {middle} {y1} {x1} {y1}");
    }
    // Over one or more columns: down or up to the gap above the lower of the two rows, across, then in.
    let lane = wide_corridor(from.row.max(to.row));
    let (leave, arrive) = ((from.column as i32 + 1) * COLUMN, to.column as i32 * COLUMN);
    format!("M{x0} {y0} C{leave} {y0} {leave} {lane} {} {lane} H{} C{arrive} {lane} {arrive} {y1} {x1} {y1}", leave + INSET, arrive - INSET)
}

/// Lays out a path. `columns` gives the number of courses of each column of the wide layout, in order;
/// courses are numbered in that order. `titled` says whether stages have titles (a flat path has none).
/// `edges` are `(prerequisite, course)` pairs of course numbers, prerequisite first.
pub fn layout(columns: &[usize], titled: bool, edges: &[(usize, usize)]) -> Layout {
    let mut slots: Vec<Slot> = columns
        .iter()
        .enumerate()
        .flat_map(|(column, &count)| (0..count).map(move |row| Slot { column, row, ..Slot::default() }))
        .collect();
    // Top of each card in the narrow layout: stage titles and cards simply follow each other.
    let mut tops = Vec::with_capacity(slots.len());
    let mut cursor = 0;
    for &count in columns {
        if titled {
            cursor += NARROW_STAGE;
        }
        for _ in 0..count {
            tops.push(cursor);
            cursor += NARROW_NODE + NARROW_GAP;
        }
    }
    let narrow_height = cursor;

    let mut edges: Vec<(usize, usize)> = edges.iter().copied().filter(|&(from, to)| from < to && to < slots.len()).collect();
    edges.sort_unstable();
    edges.dedup();

    // Rail lanes: shortest links first, each in the innermost lane that is free over its whole height.
    let span = |&(from, to): &(usize, usize)| (tops[from] + NARROW_NODE / 2 + NARROW_PORT, tops[to] + NARROW_NODE / 2 - NARROW_PORT);
    let mut by_length = edges.clone();
    by_length.sort_by_key(|edge| (edge.1 - edge.0, edge.0));
    let mut lanes: Vec<Vec<(i32, i32)>> = Vec::new();
    let mut lane_of = Vec::with_capacity(edges.len());
    for edge in &by_length {
        let (start, end) = span(edge);
        let free = |lane: &Vec<(i32, i32)>| lane.iter().all(|&(top, bottom)| end < top || bottom < start);
        let lane = lanes.iter().position(free).unwrap_or_else(|| {
            lanes.push(Vec::new());
            lanes.len() - 1
        });
        lanes[lane].push((start, end));
        lane_of.push((*edge, lane));
    }
    let rail = rail_width(lanes.len());

    let mut links = Vec::with_capacity(edges.len());
    for edge in &edges {
        let (from, to) = *edge;
        let wide = wide_path(slots[from], slots[to]);
        let same_column_next = slots[from].column == slots[to].column && slots[to].row == slots[from].row + 1;
        if same_column_next {
            slots[to].enters_top = true;
        } else {
            slots[to].enters_left = true;
        }

        let lane = lane_of.iter().find(|(other, _)| other == edge).map_or(0, |&(_, lane)| lane);
        let x = rail - LANE_START - lane as i32 * LANE;
        let (start, end) = span(edge);
        let mut narrow = String::new();
        // Out of the card, down the lane, back in: a bracket with rounded corners.
        let _ = write!(
            narrow,
            "M{rail} {start} H{} Q{x} {start} {x} {} V{} Q{x} {end} {} {end} H{}",
            x + CORNER,
            start + CORNER,
            end - CORNER,
            x + CORNER,
            rail - ARROW_LENGTH
        );
        let narrow_arrow =
            format!("{rail},{end} {},{} {},{}", rail - ARROW_LENGTH, end - ARROW_HALF, rail - ARROW_LENGTH, end + ARROW_HALF);
        links.push(Link { from, to, wide, narrow, narrow_arrow });
    }

    Layout {
        columns: columns.len(),
        slots,
        links,
        titled,
        rows: columns.iter().copied().max().unwrap_or(0),
        lanes: lanes.len(),
        narrow_height,
    }
}

fn rail_width(lanes: usize) -> i32 {
    if lanes == 0 {
        0
    } else {
        LANE_START + (lanes as i32 - 1) * LANE + RAIL_MARGIN
    }
}

impl Layout {
    fn wide_height(&self) -> i32 {
        WIDE_TOP + self.rows as i32 * WIDE_NODE + (self.rows as i32 - 1).max(0) * WIDE_GAP + WIDE_BOTTOM
    }

    fn rail(&self) -> i32 {
        rail_width(self.lanes)
    }

    /// `viewBox` of the wide SVG: stretched horizontally, exact vertically.
    pub fn wide_view_box(&self) -> String {
        format!("0 0 {} {}", self.columns.max(1) as i32 * COLUMN, self.wide_height())
    }

    /// `viewBox` of the narrow SVG, drawn at its real size.
    pub fn narrow_view_box(&self) -> String {
        format!("0 0 {} {}", self.rail(), self.narrow_height)
    }

    /// The dimensions the style sheet needs, as custom properties.
    pub fn style(&self) -> String {
        format!(
            "--pm-columns: {}; --pm-head-wide: {}; --pm-top-wide: {}; --pm-node-wide: {}; --pm-gap-wide: {}; --pm-height-wide: {}; \
             --pm-stage-narrow: {}; --pm-node-narrow: {}; --pm-gap-narrow: {}; --pm-rail: {}; --pm-height-narrow: {}",
            self.columns.max(1),
            rem(if self.titled { WIDE_HEAD } else { 0 }),
            rem(WIDE_TOP),
            rem(WIDE_NODE),
            rem(WIDE_GAP),
            rem(self.wide_height()),
            rem(NARROW_STAGE),
            rem(NARROW_NODE),
            rem(NARROW_GAP),
            rem(self.rail()),
            rem(self.narrow_height),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The numbers of an SVG path, in order.
    fn numbers(path: &str) -> Vec<i32> {
        path.split(|c: char| !c.is_ascii_digit() && c != '-').filter(|part| !part.is_empty()).map(|part| part.parse().unwrap()).collect()
    }

    #[test]
    fn courses_fill_columns_in_order() {
        let map = layout(&[2, 3, 1], true, &[]);
        let places: Vec<(usize, usize)> = map.slots.iter().map(|slot| (slot.column, slot.row)).collect();
        assert_eq!(places, [(0, 0), (0, 1), (1, 0), (1, 1), (1, 2), (2, 0)]);
        assert_eq!((map.columns, map.rows, map.lanes), (3, 3, 0));
        assert!(map.links.is_empty());
        // Three rows and their two gaps, between the top and bottom margins.
        assert_eq!(map.wide_view_box(), format!("0 0 3000 {}", 24 + 3 * 136 + 2 * 40 + 8));
        // No link, no rail; three stage titles and six cards.
        assert_eq!(map.narrow_view_box(), format!("0 0 0 {}", 3 * 44 + 6 * (128 + 14)));
    }

    #[test]
    fn dimensions_are_handed_to_the_style_sheet_in_rem() {
        assert_eq!(
            (rem(152), rem(40), rem(0), rem(44), rem(22)),
            ("9.5rem".into(), "2.5rem".into(), "0rem".into(), "2.75rem".into(), "1.375rem".into())
        );
        let style = layout(&[1, 1], true, &[(0, 1)]).style();
        assert!(style.starts_with("--pm-columns: 2; --pm-head-wide: 3rem; --pm-top-wide: 1.5rem; --pm-node-wide: 8.5rem;"), "{style}");
        assert!(style.contains("--pm-rail: 1.375rem;") && style.ends_with("--pm-height-narrow: 23.25rem"), "{style}");
        // A flat path has no stage title.
        assert!(layout(&[1, 1], false, &[]).style().contains("--pm-head-wide: 0rem;"));
    }

    #[test]
    fn a_link_to_the_next_column_stays_in_the_corridor_between_them() {
        let map = layout(&[2, 2], true, &[(1, 2), (0, 2)]);
        // Links are sorted and deduplicated.
        assert_eq!(map.links.iter().map(|link| (link.from, link.to)).collect::<Vec<_>>(), [(0, 2), (1, 2)]);
        // Same row: a straight line from the right side of one card to the left side of the other.
        assert_eq!(map.links[0].wide, "M900 92 H1100");
        // Another row: an S whose control points are on the border between the columns.
        assert_eq!(map.links[1].wide, "M900 268 C1000 268 1000 92 1100 92");
        assert!(map.slots[2].enters_left && !map.slots[2].enters_top && !map.slots[3].enters_left);
    }

    #[test]
    fn a_link_over_a_column_passes_between_its_rows() {
        let map = layout(&[2, 2, 2], true, &[(0, 4), (1, 4), (0, 5)]);
        let across = |index: usize| {
            let path = &map.links[index].wide;
            let start = path.find('H').unwrap() + 1;
            (numbers(&path[..start])[6..8].to_vec(), numbers(&path[start..])[0])
        };
        // Row 0 to row 0: above the first row, from the first corridor to the second.
        assert_eq!(across(0), (vec![1100, 12], 1900));
        // Row 0 to row 1, and row 1 to row 0: through the gap between the two rows (24 + 176 - 20).
        assert_eq!(across(1).0, [1100, 180]);
        assert_eq!(across(2).0, [1100, 180]);
        for link in &map.links {
            assert!(link.wide.ends_with(&format!("2100 {}", if link.to == 4 { 92 } else { 268 })), "{}", link.wide);
        }
    }

    #[test]
    fn links_inside_a_column_go_straight_down_or_around() {
        let map = layout(&[3], true, &[(0, 1), (0, 2)]);
        // To the card just below: from the bottom of one to the top of the other, in the middle.
        assert_eq!(map.links[0].wide, "M500 160 V200");
        assert!(map.slots[1].enters_top && !map.slots[1].enters_left);
        // Further down: out by the left side, down the left corridor, in by the left side.
        // It leaves 30 units below the middle of the first card and arrives at the middle of the third.
        assert_eq!(map.links[1].wide, "M100 122 Q33 122 33 138 V428 Q33 444 100 444");
        assert!(map.slots[2].enters_left && !map.slots[2].enters_top);
    }

    #[test]
    fn rail_links_that_overlap_get_their_own_lane() {
        // A chain shares one lane: each link ends above the middle of a card, the next starts below it.
        let chain = layout(&[4], false, &[(0, 1), (1, 2), (2, 3)]);
        assert_eq!((chain.lanes, chain.rail()), (1, 22));
        assert!(chain.links.iter().all(|link| numbers(&link.narrow)[3] == 6));
        // Out of the first card 14 units below its middle, into the second 14 units above its middle.
        assert_eq!(chain.links[0].narrow, "M22 78 H12 Q6 78 6 84 V186 Q6 192 12 192 H15");
        assert_eq!(chain.links[0].narrow_arrow, "22,192 15,188 15,196");

        // A fork and a long link: the short links take the inner lane, the long one goes around them.
        let fork = layout(&[4], false, &[(0, 1), (0, 3), (1, 2)]);
        assert_eq!((fork.lanes, fork.rail()), (2, 34));
        let lane_x =
            |from: usize, to: usize| numbers(&fork.links.iter().find(|link| (link.from, link.to) == (from, to)).unwrap().narrow)[3];
        assert_eq!((lane_x(0, 1), lane_x(1, 2), lane_x(0, 3)), (18, 18, 6));
    }

    #[test]
    fn stage_titles_shift_the_rail() {
        let titled = layout(&[1, 1], true, &[(0, 1)]);
        let flat = layout(&[1, 1], false, &[(0, 1)]);
        let ends = |map: &Layout| (numbers(&map.links[0].narrow)[1], numbers(&map.links[0].narrow_arrow)[1]);
        assert_eq!(ends(&flat), (64 + 14, 142 + 64 - 14));
        assert_eq!(ends(&titled), (44 + 64 + 14, 44 + 142 + 44 + 64 - 14));
    }

    #[test]
    fn links_that_cannot_be_drawn_are_dropped() {
        // Backwards, onto itself, or to a course that is not there.
        let map = layout(&[2], false, &[(1, 0), (1, 1), (0, 7), (0, 1), (0, 1)]);
        assert_eq!(map.links.iter().map(|link| (link.from, link.to)).collect::<Vec<_>>(), [(0, 1)]);
        assert_eq!(layout(&[], false, &[]).wide_view_box(), "0 0 1000 32");
    }
}
