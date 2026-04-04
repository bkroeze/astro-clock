//! SVG Output Integration Tests
//! Tests that verify SVG output works correctly end-to-end

use astro_clock::svg_renderer::SvgRenderer;
use astro_clock::{ChartData, GeoPos, HouseCusps, HouseSystem, PlanetPosition, Position};

fn create_test_chart_data() -> ChartData {
    ChartData {
        geo_pos: GeoPos::new(40.7128, -74.0060, 0.0),
        julian_day: 2451545.0,
        planets: vec![
            PlanetPosition::new("Sun", Position::new(280.0, 0.0, 0.0, 0.0, 0.0, 0.0), false),
            PlanetPosition::new("Moon", Position::new(45.0, 0.0, 0.0, 0.0, 0.0, 0.0), false),
            PlanetPosition::new(
                "Mercury",
                Position::new(300.0, 0.0, 0.0, 0.0, 0.0, 0.0),
                false,
            ),
            PlanetPosition::new(
                "Venus",
                Position::new(320.0, 0.0, 0.0, 0.0, 0.0, 0.0),
                false,
            ),
            PlanetPosition::new("Mars", Position::new(150.0, 0.0, 0.0, 0.0, 0.0, 0.0), true), // retrograde
        ],
        houses: HouseCusps {
            asc: 180.0,
            mc: 90.0,
            dc: 0.0,
            ic: 270.0,
            houses: [
                180.0, 210.0, 240.0, 270.0, 300.0, 330.0, 0.0, 30.0, 60.0, 90.0, 120.0, 150.0,
            ],
            system: HouseSystem::Placidus,
        },
        sidereal_time: 0.0,
    }
}

#[test]
fn test_svg_generation_basic() {
    let renderer = SvgRenderer::new(800, 800);
    let chart_data = create_test_chart_data();

    let svg = renderer.render_chart(&chart_data).unwrap();

    // Basic structure checks
    assert!(svg.contains("<svg"), "SVG should contain opening svg tag");
    assert!(svg.contains("</svg>"), "SVG should contain closing svg tag");
    assert!(
        svg.contains("xmlns=\"http://www.w3.org/2000/svg\""),
        "SVG should have xmlns"
    );
}

#[test]
fn test_svg_contains_glyph_defs() {
    let renderer = SvgRenderer::new(800, 800);
    let chart_data = create_test_chart_data();

    let svg = renderer.render_chart(&chart_data).unwrap();

    // Should contain glyph definitions
    assert!(svg.contains("<defs>"), "SVG should contain defs section");
    assert!(svg.contains("id=\"sun\""), "SVG should contain sun glyph");
    assert!(svg.contains("id=\"moon\""), "SVG should contain moon glyph");
    assert!(
        svg.contains("id=\"aries\""),
        "SVG should contain zodiac glyphs"
    );
}

#[test]
fn test_svg_no_font_dependencies() {
    let renderer = SvgRenderer::new(800, 800);
    let chart_data = create_test_chart_data();

    let svg = renderer.render_chart(&chart_data).unwrap();

    // Should NOT contain font references (except for retrograde indicators)
    // Note: The renderer uses font-family="sans-serif" for text labels
    // but should not reference external TTF files
    assert!(
        !svg.to_lowercase().contains(".ttf"),
        "SVG should not reference TTF files"
    );
    assert!(
        !svg.contains("@import"),
        "SVG should not import external resources"
    );
    assert!(
        !svg.contains("url("),
        "SVG should not reference external URLs"
    );
}

#[test]
fn test_svg_valid_xml() {
    let renderer = SvgRenderer::new(800, 800);
    let chart_data = create_test_chart_data();

    let svg = renderer.render_chart(&chart_data).unwrap();

    // Try to parse as XML using minidom
    // This will panic if XML is malformed
    let _doc: minidom::Element = svg.parse().expect("SVG should be valid XML");
}

#[test]
fn test_svg_contains_chart_elements() {
    let renderer = SvgRenderer::new(800, 800);
    let chart_data = create_test_chart_data();

    let svg = renderer.render_chart(&chart_data).unwrap();

    // Should contain chart elements
    assert!(
        svg.contains("<circle"),
        "SVG should contain circles (wheel)"
    );
    assert!(
        svg.contains("<line"),
        "SVG should contain lines (cusps/aspects)"
    );
    assert!(
        svg.contains("<use"),
        "SVG should contain use elements (glyphs)"
    );
}

#[test]
fn test_svg_retrograde_indicator() {
    let renderer = SvgRenderer::new(800, 800);
    let chart_data = create_test_chart_data();

    let svg = renderer.render_chart(&chart_data).unwrap();

    // Mars is retrograde in test data - check for retrograde indicator
    // The renderer uses a 'r' text element for retrograde indicators
    assert!(
        svg.contains("<text") && svg.contains(">r<"),
        "SVG should indicate retrograde planets with 'r' text element"
    );
}

#[test]
fn test_svg_has_correct_dimensions() {
    let renderer = SvgRenderer::new(800, 800);
    let chart_data = create_test_chart_data();

    let svg = renderer.render_chart(&chart_data).unwrap();

    // Check dimensions are in the output
    assert!(
        svg.contains(r#"width="800""#),
        "SVG should have correct width"
    );
    assert!(
        svg.contains(r#"height="800""#),
        "SVG should have correct height"
    );
    assert!(
        svg.contains(r#"viewBox="0 0 800 800""#),
        "SVG should have correct viewBox"
    );
}

#[test]
fn test_svg_contains_all_planet_glyphs() {
    let renderer = SvgRenderer::new(800, 800);
    let chart_data = create_test_chart_data();

    let svg = renderer.render_chart(&chart_data).unwrap();

    // Check that all planets in the chart data have corresponding glyph definitions
    let expected_glyphs = ["sun", "moon", "mercury", "venus", "mars"];
    for glyph in &expected_glyphs {
        assert!(
            svg.contains(&format!(r#"id="{}""#, glyph)),
            "SVG should contain {} glyph definition",
            glyph
        );
    }
}

#[test]
fn test_svg_contains_zodiac_sign_glyphs() {
    let renderer = SvgRenderer::new(800, 800);
    let chart_data = create_test_chart_data();

    let svg = renderer.render_chart(&chart_data).unwrap();

    // Check that all zodiac sign glyphs are defined
    let zodiac_signs = [
        "aries",
        "taurus",
        "gemini",
        "cancer",
        "leo",
        "virgo",
        "libra",
        "scorpio",
        "sagittarius",
        "capricorn",
        "aquarius",
        "pisces",
    ];

    for sign in &zodiac_signs {
        assert!(
            svg.contains(&format!(r#"id="{}""#, sign)),
            "SVG should contain {} zodiac glyph",
            sign
        );
    }
}

#[test]
fn test_svg_has_no_external_references() {
    let renderer = SvgRenderer::new(800, 800);
    let chart_data = create_test_chart_data();

    let svg = renderer.render_chart(&chart_data).unwrap();

    // Check that all content is embedded (no external file references)
    assert!(
        !svg.contains("xlink:href"),
        "SVG should not use xlink:href (deprecated)"
    );

    // href in <use> is OK as long as it references internal IDs (starting with #)
    let use_hrefs: Vec<_> = svg.matches("href=").collect();
    for href in use_hrefs {
        // Each use href should be followed by a # (internal reference)
        let idx = svg.find(href).unwrap();
        let after_href = &svg[idx..idx + href.len() + 10];
        assert!(
            after_href.contains("#"),
            "All href references should be internal (start with #)"
        );
    }
}

#[test]
fn test_svg_save_and_load() {
    let renderer = SvgRenderer::new(800, 800);
    let chart_data = create_test_chart_data();

    // Create a temp file
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path().join("test_chart.svg");

    // Save the SVG
    renderer
        .save(&chart_data, temp_path.to_str().unwrap())
        .unwrap();

    // Verify file exists and can be read
    assert!(temp_path.exists(), "SVG file should be saved");

    let loaded_svg = std::fs::read_to_string(&temp_path).unwrap();
    assert!(loaded_svg.contains("<svg"), "Saved SVG should be valid");

    // Verify it's the same content
    let rendered_svg = renderer.render_chart(&chart_data).unwrap();
    assert_eq!(
        loaded_svg, rendered_svg,
        "Saved SVG should match rendered SVG"
    );
}

#[test]
fn test_svg_different_sizes() {
    let chart_data = create_test_chart_data();

    // Test various sizes
    let sizes = [(400, 400), (800, 600), (1200, 1200), (1920, 1080)];

    for (width, height) in &sizes {
        let renderer = SvgRenderer::new(*width, *height);
        let svg = renderer.render_chart(&chart_data).unwrap();

        // Verify dimensions are correct
        assert!(
            svg.contains(&format!(r#"width="{}""#, width)),
            "SVG should have width {} for {}x{} renderer",
            width,
            width,
            height
        );
        assert!(
            svg.contains(&format!(r#"height="{}""#, height)),
            "SVG should have height {} for {}x{} renderer",
            height,
            width,
            height
        );

        // Verify XML is valid for each size
        let _: minidom::Element = svg.parse().expect(&format!(
            "SVG for size {}x{} should be valid XML",
            width, height
        ));
    }
}

#[test]
fn test_svg_contains_house_labels() {
    let renderer = SvgRenderer::new(800, 800);
    let chart_data = create_test_chart_data();

    let svg = renderer.render_chart(&chart_data).unwrap();

    // Check for text elements (house labels)
    assert!(
        svg.contains("<text"),
        "SVG should contain text elements for house labels"
    );

    // The renderer uses zodiac symbols and degree notation
    assert!(
        svg.contains("°"),
        "SVG should contain degree symbols in labels"
    );
    assert!(
        svg.contains("'"),
        "SVG should contain minute symbols in labels"
    );
}
