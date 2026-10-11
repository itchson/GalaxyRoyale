//! Original 64-cell workshop parts: conductor, shield lattice and annular bearing.
//!
//! Source cells are cached without resizing. The UI consumer owns continuous fit,
//! clipping and whole-element budget admission; these bodies carry no game state.
use engine_core::PixelBody;
use std::sync::LazyLock;

const SIZE: usize = 64;
const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    u32::from_le_bytes([r, g, b, 255])
}
const PALETTE: [(char, u32); 16] = [
    ('k', rgb(13, 22, 29)),
    ('d', rgb(30, 42, 49)),
    ('s', rgb(68, 83, 89)),
    ('p', rgb(140, 157, 157)),
    ('w', rgb(217, 227, 219)),
    ('a', rgb(184, 176, 148)),
    ('c', rgb(139, 84, 52)),
    ('r', rgb(112, 41, 37)),
    ('R', rgb(218, 86, 66)),
    ('!', rgb(255, 157, 118)),
    ('b', rgb(24, 65, 98)),
    ('B', rgb(63, 143, 191)),
    ('^', rgb(144, 208, 229)),
    ('g', rgb(24, 64, 39)),
    ('G', rgb(79, 155, 88)),
    ('*', rgb(156, 224, 173)),
];

const PULSE: [&str; SIZE] = [
    "                                                                ",
    "                                                                ",
    "                                                                ",
    "                                                                ",
    "                                                                ",
    "                                                                ",
    "                                                                ",
    "                                                                ",
    "                                                                ",
    "                                                                ",
    "               wwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwww            ",
    "              pppppppppppppppppppppppppppppppppppppw            ",
    "             psssssssssssssssssssssssssssssssssssspw            ",
    "            wpsswkddddddddddddddddddddddddddddswkspw            ",
    "            wpssssddddddddddddddddddddddddddddsssspw            ",
    "            wpsdddkkkkkkkkkkkkkkkkkkkkkkkkkkkkdddspw            ",
    "            wpsdddkkkkkkkkkkkkkkkkkkkkkkkkkkkkdddspw            ",
    "            wpsdddkkkppppkkppppkkppppkkppppkkkdddspw            ",
    "     pppppppppsdddkkkppppkkppppkkppppkkppppkkkdRrsppppppppp     ",
    "     pppppppppkkkdkkk!RRckk!RRckk!RRckk!RRckkkdrrsppppppppp     ",
    "     saaaaaasskkkdkkk!RRckk!RRckk!RRckk!RRckkkdrrsssaaaaaas     ",
    "     ssssssssssdddkkk!RRckk!RRckk!RRckk!RRckkkdddssssssssss     ",
    "     kkkkkkkkksdddkkk!RRckk!RRckk!RRckk!RRckkkdddskkkkkkkkk     ",
    "            wpsdddkkc!Rcccc!Rcccc!Rcccc!Rccckkdddspw            ",
    "            wpsdddkkcccccccccccccccccccccccckkdddspw            ",
    "            wpkkkdkkk!RRckk!RRckk!RRckk!RRckkkdRrspw            ",
    "            wpkkkdkkk!RRckk!RRckk!RRckk!RRckkkdrrspw            ",
    "            wpsdddkkk!RRckk!RRckk!RRckk!RRckkkdrrspw            ",
    "            wpsdddkkk!RRckk!RRckk!RRckk!RRckkkdddspw            ",
    "     pppppppppsdddkkk!RRckk!RRckk!RRckk!RRckkkdddsppppppppp     ",
    "     pppppppppsdddkkk!RRckk!RRckk!RRckk!RRckkkdddsppppppppp     ",
    "     saaaaaasskkkdkkk!RRckk!RRckk!RRckk!RRckkkdddsssaaaaaas     ",
    "     ssssssssskkkdkkk!RRckk!RRckk!RRckk!RRckkkdRrssssssssss     ",
    "     kkkkkkkkksdddkkk!RRckk!RRckk!RRckk!RRckkkdrrskkkkkkkkk     ",
    "            wpsdddkkc!Rcccc!Rcccc!Rcccc!Rccckkdrrspw            ",
    "            wpsdddkkcccccccccccccccccccccccckkdddspw            ",
    "            wpsdddkkk!RRckk!RRckk!RRckk!RRckkkdddspw            ",
    "            wpkkkdkkk!RRckk!RRckk!RRckk!RRckkkdddspw            ",
    "            wpkkkdkkk!RRckk!RRckk!RRckk!RRckkkdddspw            ",
    "            wpsdddkkk!RRckk!RRckk!RRckk!RRckkkdRrspw            ",
    "     pppppppppsdddkkk!RRckk!RRckk!RRckk!RRckkkdrrsppppppppp     ",
    "     pppppppppsdddkkk!RRckk!RRckk!RRckk!RRckkkdrrsppppppppp     ",
    "     saaaaaasssdddkkc!Rcccc!Rcccc!Rcccc!Rccckkdddsssaaaaaas     ",
    "     ssssssssskkkdkkcccccccccccccccccccccccckkdddssssssssss     ",
    "     kkkkkkkkkkkkdkkk!RRckk!RRckk!RRckk!RRckkkdddskkkkkkkkk     ",
    "            wpsdddkkk!RRckk!RRckk!RRckk!RRckkkdddspw            ",
    "            wpsdddkkksssskksssskksssskksssskkkdRrspw            ",
    "            wpsdddkkksssskksssskksssskksssskkkdrrspw            ",
    "            wpsdddkkkkkkkkkkkkkkkkkkkkkkkkkkkkdrrspw            ",
    "            wpsdddkkkkkkkkkkkkkkkkkkkkkkkkkkkkdddspw            ",
    "            wpssssddddddddddddddddddddddddddddsssspw            ",
    "            wpsswkddddddddddddddddddddddddddddswkspw            ",
    "            wpssssssssssssssssssssssssssssssssssssp             ",
    "            wppppppppppppppppppppppppppppppppppppp              ",
    "            wwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwww               ",
    "                      ssppppppppppppppppss                      ",
    "                      ssppccpppccpppccppss                      ",
    "                      ssssccsssccsssccssss                      ",
    "                      ssssccsssccsssccssss                      ",
    "                                                                ",
    "                                                                ",
    "                                                                ",
    "                                                                ",
    "                                                                ",
];

const AEGIS: [&str; SIZE] = [
    "                                                                ",
    "                                                                ",
    "                                                                ",
    "                                                                ",
    "                                                                ",
    "                                                                ",
    "                                                                ",
    "                                                                ",
    "              wwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwww             ",
    "             ppppppppppppppppppppppppppppppppppppppp            ",
    "            sssssssssssssssssssssssssssssssssssssssss           ",
    "           psddddddddddpppppppppppppppppppddddddddddsp          ",
    "          wpssssdd^bdd^bpppwwwwwwwwwwwpppp^bdd^bdsssspw         ",
    "          wpsswkddbbddbbpppwwwwwwwwwwwppppbbddbbdswkspw         ",
    "          wpssssddbbddbbddddddddddddddddddbbddbbdsssspw         ",
    "          wpsddkBbbbkkbbBbkkBbkkkkkkBbkkBbbbkkbbBbddspw         ",
    "          wpsddkkBbbkkbbbkkkkBbkkkkBbkkkkBbbkkbbbkddspw         ",
    "          wpsddkkkbbkkbbkkkkkkBbkkBbkkkkkkbbkkbbkkddspw         ",
    "          wpsddkkkkBbBbkkkkkkkkBbBbkkkkkkkkBbBbkkkddspw         ",
    "          wpsddkkkkk^bkkkkkkkkkk^bkkkkkkkkkk^bkkkkddspw         ",
    "          wpsddkkkkBbBbkkkkkkkkBbBbkkkkkkkkBbBbkkkddspw         ",
    "          wpsddkkkBbkkBbkkkkkkBbkkBbkkkkkkBbkkBbkkddspw         ",
    "          wpsddkkBbkkkkBbkkkkBbkkkkBbkkkkBbkkkkBbkddspw         ",
    "          wpbBbkBbkkkkkkBbkkBbkkkkkkBbkkBbkkkkkkBbbBbpw         ",
    "          wpbbbBbkkkkkkkkBbBbkkkkkkkkBbBbkkkkkkkkBbbbpw         ",
    "          wpbbbbkkkkkkkkkk^bkkkkkkkkkk^bkkkkkkkkkkbbbpw         ",
    "          wpsddBbkkkkkkkkBbBbkkkkkkkkBbBbkkkkkkkkBddspw         ",
    "          wpsddkBbkkkkkkBbkkBbkkkkkkBbkkBbkkkkkkBbddspw         ",
    "          wpsddkkBbkkkkBbkkkkBbkkkkBbkkkkBbkkkkBbkddspw         ",
    "          wpbBbkkkBbkkBbkkkkkkBbkkBbkkkkkkBbkkBbkkbBbpw         ",
    "          wpbbbkkkkBbBbkkkkkkkkBbBbkkkkkkkkBbBbkkkbbbpw         ",
    "          wpbbbkkkkk^bkkkkkkkkkk^bkkkkkkkkkk^bkkkkbbbpw         ",
    "          wpsddkkkkBbBbkkkkkkkkBbBbkkkkkkkkBbBbkkkddspw         ",
    "          wpsddkkkBbkkBbkkkkkkBbkkBbkkkkkkBbkkBbkkddspw         ",
    "          wpsddkkBbkkkkBbkkkkBbkkkkBbkkkkBbkkkkBbkddspw         ",
    "          wpbBbkssskkkkkBbkkBbkkkkkkBbkkBbkkkksssbbBbpw         ",
    "          wpbbbdswkkkkkkkBbBbkkkkkkkkBbBbkkkkkswkdbbbpw         ",
    "          wpbbbdssskkkkkkk^bkkkkkkkkkk^bkkkkkksssdbbbpw         ",
    "           wpsddddkkkkkkkBbBbkkkkkkkkBbBbkkkkkkddddspw          ",
    "            wpsddddkkkkkBbkkBbkkkkkkBbkkBbkkkkddddspw           ",
    "             wpsddddkkkBbkkkkBbkkkkBbkkkkBbkkddddspw            ",
    "              wpsddddkBbkkkkkkBbkkBbkkkkkkBbddddspw             ",
    "               wpsddddbkkkkkkkkBbBbkkkkkkkkddddspw              ",
    "                wpsddddkkkkkkkkk^bkkkkkkkkddddspw               ",
    "                 wpsddddkkkkkkkBbBbkkkkkkddddspw                ",
    "                  wpsddddkkkkkBbkkBbkkkkddddspw                 ",
    "                   wpsddddkkkBbkkkkBbkkddddspw                  ",
    "                    wpsddddkBbkkkkkkBbddddspw                   ",
    "                     wpsddddbkkkkkkkkddddspw                    ",
    "                      wpsddddkkkkkkkddddspw                     ",
    "                       wpsdddddddddddddspw                      ",
    "                        wpsdddddddddddspw                       ",
    "                         wpsdddddddddspw                        ",
    "                          wpsdddddddspw                         ",
    "                           wpsddBddspw                          ",
    "                            wpsd^dspw                           ",
    "                             wpsBspw                            ",
    "                              wpppw                             ",
    "                               www                              ",
    "                                                                ",
    "                                                                ",
    "                                                                ",
    "                                                                ",
    "                                                                ",
];

const FLUX: [&str; SIZE] = [
    "                                                                ",
    "                                                                ",
    "                                                                ",
    "                           ssssssssss                           ",
    "                           sppsssppps                           ",
    "                           sppswkppps                           ",
    "                           sppsssppps                           ",
    "                       wwwwsssssssssswwww                       ",
    "                     wwwwwwppppppppppwwwwww                     ",
    "                   wwwwwppppppppppppppppwwwww                   ",
    "                 wwwwppppppsssspppsssppppppwwww                 ",
    "                wwwwppppsssssspwppssssssppppwwww                ",
    "               wwwppppsssssskkppasskssssssppppwww               ",
    "              wwwpppsssskkkkkkppssskkkkksssspppwww              ",
    "             wwwpppssskkkkkkkkkssskkkkkkkkssspppwss             ",
    "            wwwppsssskkkkkkkddddddddkkkkkkkssssppsss            ",
    "           wwwppspppkkkkkddd**dddddddddkkkkkkpppppsss           ",
    "          wwwppspwppskkdd*GGggdddddddddddkkkpwppsppsss          ",
    "          wwpppsppasskd*Ggggppppppppddddddkkppasspppss          ",
    "         wwwppssppsssd*Gggppppppppppppddddddppssssppsss         ",
    "         wwppsssksssdddgpppppGGGGGGpppppdddddssssssppss         ",
    "        wwpppsskkkkddddpppGGGGGGGGGGGGpppddddkkkksspppss        ",
    "        wwppsskkkkddddpppGGGGggggggGGGGppsdd*dkkkkssppss        ",
    "       wwwppsskkkddddpppGGGgg      ggGGGsssgG*dkkkssppsss       ",
    "       wwppsskkkkdddpppGGgg          ggGGsssgGdkkkkssppss       ",
    "       wwppsskkkddddppGGgg            ggGGssgg*dkkkssppss       ",
    "       wwppsskkkdddppGGGg              gGGGssgGdkkkssppss       ",
    "   sssssppssskkkdddppGGg                gGGssgGdkkksssppsssss   ",
    "   spppsppsskkkdddpppGGg                gGGsssg*dkkkssppsppps   ",
    "   spppsppsskkkdddppGGg                  gGGssg*dkkkssppsppps   ",
    "   sssssppspppkdddppGGg                  gGGssdddkpppsppsssss   ",
    "   sswkspppwppsdddppGGg                  gGGssdddpwppsppsswks   ",
    "   sssssppppassdddppGGg                  gGGssdddppassppsssss   ",
    "   spppsppppsssdddppGGg                  gGGssdddppsssppsppps   ",
    "   spppsppsssskd*gppGGg                  gGGssdddkssssppsppps   ",
    "   spppsppsskkkd*gpppGGg                gGGsssdddkkkssppsppps   ",
    "   sssssppssskkkdGgppGGg                gGGssdddkkksssppsssss   ",
    "       wwppsskkkdGgppGGGg              gGGGssdddkkkssppss       ",
    "       wwppsskkkd*ggppGGgg            ggGGssddddkkkssppss       ",
    "       wwppsskkkkdGgpppGGgg          ggGGsssdddkkkkssppss       ",
    "       wwwppsskkkd*GgppsGGGgg      ggGGGsssddddkkkssppsss       ",
    "        wwppsskkkkd*ddsssGGGGggggggGGGGsssddddkkkkssppss        ",
    "        wwpppsskkkkddddsssGGGGGGGGGGGGsssddddkkkksspppss        ",
    "         wwppssskkkdddddsssssGGGGGGsssssgddddkkksssppss         ",
    "         wwwppsskpppddddddssssssssssssggG*ddkpppssppsss         ",
    "          wwpppspwppskddddddssssssssgggG*dkkpwppspppss          ",
    "          wwwppsppasskkdddddddddddggGG*ddkkkppassppsss          ",
    "           wwwppppssskkkkddddddddd**dddkkkkkppssspsss           ",
    "            wwwppsssskkkkkkkddddddddkkkkkkkssssspsss            ",
    "             wsspppssskkkkkkkkkpppkkkkkkkkssspppsss             ",
    "              ssspppsssskkkkkkpwppskkkkksssspppsss              ",
    "               sssppppsssssskkppasskssssssppppsss               ",
    "                ssssppppssssssppssssssssppppssss                ",
    "                 ssssppppppssssssssssppppppssss                 ",
    "                   sssssppppppppppppppppsssss                   ",
    "                     ssssssppppppppppssssss                     ",
    "                       ssssssssssssssssss                       ",
    "                           sppsssppps                           ",
    "                           sppswkppps                           ",
    "                           sppsssppps                           ",
    "                           ssssssssss                           ",
    "                                                                ",
    "                                                                ",
    "                                                                ",
];

static BODIES: LazyLock<[PixelBody; 3]> = LazyLock::new(|| {
    [PULSE, AEGIS, FLUX].map(|rows| {
        assert!(rows.iter().all(|row| row.len() == SIZE));
        assert!(
            rows.iter()
                .flat_map(|row| row.chars())
                .all(|c| { c == ' ' || PALETTE.iter().any(|&(key, _)| key == c) })
        );
        let body = PixelBody::from_ascii(&rows, &PALETTE);
        assert!((500..3000).contains(&body.occupied_count()));
        body
    })
});

/// Authoritative catalog order: 0 Pulse Conductor, 1 Aegis Lattice, 2 Flux Bearing.
/// Invalid categories have no placeholder or fallback art.
pub(crate) fn body(category: usize) -> Option<&'static PixelBody> {
    BODIES.get(category)
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_core::{AffineTransform, FrameBuilder, Transform, Vec2};
    use std::collections::{HashSet, VecDeque};

    #[test]
    fn catalog_bodies_are_cached_opaque_bounded_and_exactly_authored() {
        let expected = [2078, 1813, 1992];
        for (category, count) in expected.into_iter().enumerate() {
            let art = body(category).unwrap();
            assert_eq!((art.width, art.height), (64, 64));
            assert_eq!(art.occupied_count(), count);
            assert!(std::ptr::eq(art, body(category).unwrap()));
            assert!(art.cells().all(|(_, c)| c >> 24 == 255));
            assert!(
                art.cells()
                    .all(|(_, c)| PALETTE.iter().any(|&(_, p)| p == c))
            );
            // Empty margin keeps occupied corners inside the measured source field.
            for n in 0..64 {
                assert!(!art.is_occupied(n, 0) && !art.is_occupied(n, 63));
                assert!(!art.is_occupied(0, n) && !art.is_occupied(63, n));
            }
        }
        assert!(body(3).is_none() && body(usize::MAX).is_none());
    }

    #[test]
    fn meaningful_mechanical_landmarks_and_axle_aperture_survive_import() {
        let pulse = body(0).unwrap();
        assert_eq!(pulse.color_at(6, 20), Some(rgb(184, 176, 148))); // connector tooth
        assert_eq!(pulse.color_at(21, 30), Some(rgb(255, 157, 118))); // primary rail
        assert_eq!(pulse.color_at(19, 16), Some(rgb(13, 22, 29))); // dark well
        assert_eq!(pulse.color_at(26, 58), Some(rgb(139, 84, 52))); // lower contact
        let aegis = body(1).unwrap();
        assert_eq!(aegis.color_at(32, 12), Some(rgb(217, 227, 219))); // plated crown
        assert_eq!(aegis.color_at(32, 31), Some(rgb(144, 208, 229))); // lattice crossing
        assert_eq!(aegis.color_at(32, 55), Some(rgb(144, 208, 229))); // tapered node
        assert!(!aegis.is_occupied(12, 55)); // silhouette narrows toward base
        let flux = body(2).unwrap();
        assert_eq!(flux.color_at(32, 12), Some(rgb(184, 176, 148))); // raceway ball
        assert_eq!(flux.color_at(31, 5), Some(rgb(217, 227, 219))); // housing fastener
        for y in 27..=36 {
            for x in 27..=36 {
                assert!(!flux.is_occupied(x, y), "axle hole {x},{y}");
            }
        }
    }

    #[test]
    fn fixed_corporate_accents_do_not_recolor_the_neutral_material_family() {
        let families = [
            [rgb(112, 41, 37), rgb(218, 86, 66), rgb(255, 157, 118)],
            [rgb(24, 65, 98), rgb(63, 143, 191), rgb(144, 208, 229)],
            [rgb(24, 64, 39), rgb(79, 155, 88), rgb(156, 224, 173)],
        ];
        for category in 0..3 {
            let colors: HashSet<_> = body(category).unwrap().cells().map(|(_, c)| c).collect();
            assert!(families[category].iter().all(|c| colors.contains(c)));
            for (other, family) in families.iter().enumerate() {
                if category != other {
                    assert!(family.iter().all(|c| !colors.contains(c)));
                }
            }
            assert!(colors.contains(&rgb(13, 22, 29)) && colors.contains(&rgb(140, 157, 157)));
            assert!(colors.contains(&rgb(217, 227, 219)));
        }
    }

    #[test]
    fn casings_are_connected_and_three_silhouettes_are_distinct() {
        let masks: Vec<_> = (0..3)
            .map(|category| {
                let art = body(category).unwrap();
                let occupied: HashSet<_> = art.cells_indexed().map(|(i, _, _)| i).collect();
                let first = *occupied.iter().next().unwrap();
                let mut reached = HashSet::from([first]);
                let mut queue = VecDeque::from([first]);
                while let Some(i) = queue.pop_front() {
                    let (x, y) = (i % SIZE, i / SIZE);
                    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        let (xx, yy) = (x as isize + dx, y as isize + dy);
                        if xx < 0 || yy < 0 || xx >= SIZE as isize || yy >= SIZE as isize {
                            continue;
                        }
                        let n = yy as usize * SIZE + xx as usize;
                        if occupied.contains(&n) && reached.insert(n) {
                            queue.push_back(n);
                        }
                    }
                }
                assert_eq!(
                    reached.len(),
                    occupied.len(),
                    "disconnected category {category}"
                );
                occupied
            })
            .collect();
        for a in 0..3 {
            for b in a + 1..3 {
                assert!(masks[a].symmetric_difference(&masks[b]).count() > 500);
            }
        }
    }

    #[test]
    fn full_source_cells_extract_at_fractional_fit_under_nested_reflected_affine() {
        let parent = AffineTransform {
            position: Vec2::new(110.25, 79.5),
            basis_x: Vec2::new(-1.2, 0.35),
            basis_y: Vec2::new(0.45, 0.8),
        };
        for category in 0..3 {
            let art = body(category).unwrap();
            for fit in [48.0 / 64.0, 14.0 / 64.0] {
                let local = Transform {
                    position: Vec2::new(3.25, -5.5),
                    rotation: 0.37,
                    scale: Vec2::splat(fit),
                }
                .affine();
                let transform = parent.compose(local);
                let mut builder = FrameBuilder::new();
                builder.push_body_affine(art, transform);
                let frame = builder.finish();
                assert_eq!(frame.pixel_count, art.occupied_count());
                assert_eq!(frame.batches.len(), 1);
                let batch = &frame.batches[0];
                assert_eq!(batch.transforms.len(), 1);
                assert_eq!(
                    [batch.transforms[0].b[2], batch.transforms[0].b[3]],
                    [0.0, 0.0]
                );
                for (pixel, (expected_local, color)) in batch.pixels.iter().zip(art.cells()) {
                    assert_eq!(pixel.local, expected_local.to_array());
                    assert_eq!(pixel.color, color);
                    for corner in [
                        Vec2::new(-0.5, -0.5),
                        Vec2::new(0.5, -0.5),
                        Vec2::new(0.5, 0.5),
                        Vec2::new(-0.5, 0.5),
                    ] {
                        let expected = transform.point(expected_local + corner);
                        let actual = batch.transforms[pixel.transform as usize]
                            .point(expected_local + corner);
                        assert!(actual.is_finite() && actual.distance(expected) < 0.0001);
                    }
                }
            }
        }
    }
}
