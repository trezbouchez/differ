use crate::slicer::Chunk;
use std::fmt::{Debug, Display, Formatter, Result};
use std::ops::Range;

#[derive(Debug, PartialEq)]
pub enum Segment {
    Old(Range<usize>),
    New(Range<usize>),
}

impl Display for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Segment::Old(range) => { write!(f, "OLD[{}..{}]", range.start, range.end) },
            Segment::New(range) => { write!(f, "NEW[{}..{}]", range.start, range.end) },
        }
    }
}

pub(crate) fn delta(chunks_old: &[Chunk], chunks_new: &[Chunk], lcs: &[String]) -> Vec<Segment> {
    if lcs.is_empty() {
        return if let Some(last_new_chunk) = chunks_new.last() {
            vec![Segment::New(0..last_new_chunk.end)]
        } else {
            Vec::new()
        };
    }

    let mut segments: Vec<Segment> = Vec::with_capacity(chunks_new.len());
    let mut new_pos: usize = 0;
    let mut old_pos: usize = 0;
    let mut lcs_pos: usize = 0;
    let mut common_chunk_hash = &lcs[lcs_pos];
    let lcs_len = lcs.len();

    while lcs_pos < lcs_len {
        // Create concatenated New segment (if any)
        let new_segment_start = new_pos;
        while chunks_new[new_pos].hash != *common_chunk_hash {
            new_pos += 1;
        }
        if new_pos != new_segment_start {
            let segment_start = if new_segment_start == 0 {
                0
            } else {
                chunks_new[new_segment_start - 1].end
            };
            let new_segment = Segment::New(segment_start..chunks_new[new_pos - 1].end);
            segments.push(new_segment);
        }

        // Skip deleted old region
        while chunks_old[old_pos].hash != *common_chunk_hash {
            old_pos += 1;
        }

        // Create concatenated Old segment
        let old_segment_start = old_pos;
        while chunks_new[new_pos].hash == *common_chunk_hash
            && chunks_old[old_pos].hash == *common_chunk_hash
        {
            new_pos += 1;
            old_pos += 1;
            lcs_pos += 1;
            if lcs_pos == lcs_len {
                break;
            }
            common_chunk_hash = &lcs[lcs_pos];
        }
        if old_pos != old_segment_start {
            let segment_start = if old_segment_start == 0 {
                0
            } else {
                chunks_old[old_segment_start - 1].end
            };
            let old_segment = Segment::Old(segment_start..chunks_old[old_pos - 1].end);
            segments.push(old_segment);
        }
    }

    // Append remaining New segment
    if new_pos < chunks_new.len() {
        let segment_start = if new_pos == 0 {
            0
        } else {
            chunks_new[new_pos - 1].end
        };
        let new_segment = Segment::New(segment_start..chunks_new.last().unwrap().end);
        segments.push(new_segment);
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_nothing_in_common() {
        let old_chunks: &[Chunk] = &[Chunk {
            hash: "A".to_string(),
            end: 4,
        }];

        let new_chunks: &[Chunk] = &[Chunk {
            hash: "V".to_string(),
            end: 4,
        }];
        let lcs: &[String] = &[];
        let segments = delta(old_chunks, new_chunks, lcs);
        assert_eq!(segments, vec![Segment::New(0..4)]);
    }

    #[test]
    fn test_delta_empty_new() {
        let old_chunks: &[Chunk] = &[Chunk {
            hash: "A".to_string(),
            end: 4,
        }];

        let new_chunks: &[Chunk] = &[];

        let lcs: &[String] = &[];
        let segments = delta(old_chunks, new_chunks, lcs);
        assert_eq!(segments, vec![]);
    }

    #[test]
    fn test_delta_empty_old() {
        let old_chunks: &[Chunk] = &[];

        // single
        let new_chunks: &[Chunk] = &[Chunk {
            hash: "V".to_string(),
            end: 4,
        }];
        let lcs: &[String] = &[];
        let segments = delta(old_chunks, new_chunks, lcs);
        assert_eq!(segments, vec![Segment::New(0..4)]);

        // many
        let new_chunks: &[Chunk] = &[
            Chunk {
                hash: "V".to_string(),
                end: 4,
            },
            Chunk {
                hash: "W".to_string(),
                end: 8,
            },
        ];
        let lcs: &[String] = &[];
        let segments = delta(old_chunks, new_chunks, lcs);
        assert_eq!(segments, vec![Segment::New(0..8)]);
    }

    #[test]
    fn test_delta_empty_both() {
        let old_chunks: &[Chunk] = &[];
        let new_chunks: &[Chunk] = &[];
        let lcs: &[String] = &[];
        let segments = delta(old_chunks, new_chunks, lcs);
        assert_eq!(segments, vec![]);
    }
    #[test]
    fn test_delta_prepend() {
        let old_chunks: &[Chunk] = &[Chunk {
            hash: "A".to_string(),
            end: 4,
        }];

        // prepend one
        let new_chunks: &[Chunk] = &[
            Chunk {
                hash: "V".to_string(),
                end: 4,
            },
            Chunk {
                hash: "A".to_string(),
                end: 8,
            },
        ];
        let lcs: &[String] = &["A".to_string()];
        let segments = delta(old_chunks, new_chunks, lcs);
        assert_eq!(segments, vec![Segment::New(0..4), Segment::Old(0..4),]);

        // prepend multiple
        let new_chunks: &[Chunk] = &[
            Chunk {
                hash: "V".to_string(),
                end: 4,
            },
            Chunk {
                hash: "W".to_string(),
                end: 8,
            },
            Chunk {
                hash: "A".to_string(),
                end: 12,
            },
        ];
        let lcs: &[String] = &["A".to_string()];
        let segments = delta(old_chunks, new_chunks, lcs);
        assert_eq!(segments, vec![Segment::New(0..8), Segment::Old(0..4),]);
    }

    #[test]
    fn test_delta_append() {
        let old_chunks: &[Chunk] = &[Chunk {
            hash: "A".to_string(),
            end: 4,
        }];

        // append one
        let new_chunks: &[Chunk] = &[
            Chunk {
                hash: "A".to_string(),
                end: 4,
            },
            Chunk {
                hash: "V".to_string(),
                end: 8,
            },
        ];
        let lcs: &[String] = &["A".to_string()];
        let segments = delta(old_chunks, new_chunks, lcs);
        assert_eq!(segments, vec![Segment::Old(0..4), Segment::New(4..8),]);

        // append multiple
        let new_chunks: &[Chunk] = &[
            Chunk {
                hash: "A".to_string(),
                end: 4,
            },
            Chunk {
                hash: "V".to_string(),
                end: 8,
            },
            Chunk {
                hash: "X".to_string(),
                end: 12,
            },
        ];
        let lcs: &[String] = &["A".to_string()];
        let segments = delta(old_chunks, new_chunks, lcs);
        assert_eq!(segments, vec![Segment::Old(0..4), Segment::New(4..12)]);
    }

    #[test]
    fn test_delta_insert() {
        let old_chunks: &[Chunk] = &[
            Chunk {
                hash: "A".to_string(),
                end: 4,
            },
            Chunk {
                hash: "B".to_string(),
                end: 8,
            },
        ];

        // insert one
        let new_chunks: &[Chunk] = &[
            Chunk {
                hash: "A".to_string(),
                end: 4,
            },
            Chunk {
                hash: "V".to_string(),
                end: 8,
            },
            Chunk {
                hash: "B".to_string(),
                end: 12,
            },
        ];
        let lcs: &[String] = &["A".to_string(), "B".to_string()];
        let segments = delta(old_chunks, new_chunks, lcs);
        assert_eq!(
            segments,
            vec![Segment::Old(0..4), Segment::New(4..8), Segment::Old(4..8)]
        );

        // insert multiple
        let new_chunks: &[Chunk] = &[
            Chunk {
                hash: "A".to_string(),
                end: 4,
            },
            Chunk {
                hash: "V".to_string(),
                end: 8,
            },
            Chunk {
                hash: "W".to_string(),
                end: 12,
            },
            Chunk {
                hash: "X".to_string(),
                end: 16,
            },
            Chunk {
                hash: "B".to_string(),
                end: 20,
            },
        ];
        let lcs: &[String] = &["A".to_string(), "B".to_string()];
        let segments = delta(old_chunks, new_chunks, lcs);
        assert_eq!(
            segments,
            vec![Segment::Old(0..4), Segment::New(4..16), Segment::Old(4..8)]
        );
    }
}
