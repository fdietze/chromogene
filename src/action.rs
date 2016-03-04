use palette::Lab;
use fitness::*;
use fitness::Parameter::*;
use fitness::Stat::*;
use fitness::TargetDirection::*;

#[derive(PartialEq, Debug)]
pub enum Action {
    SetFreeColorCount(usize),
    SetPresetColors(Vec<Lab>),
    SetFixedColors(Vec<Lab>),
    SetTarget(Target),
    RemoveTarget((Stat, Parameter)),
}

pub enum ColorFilter {
    Redshift,
    Deuter,
}


pub fn line_to_action(line: &str) -> Result<Action, &str> {
    let mut words = line.trim().split(" ");
    words.next().ok_or("expected action").and_then(|string| {
        match string {
            "target" => line_to_target(line).map(|t| Action::SetTarget(t)),
            "freecolorcount" => {
                words.next()
                     .ok_or("expected string")
                     .and_then(|s| {
                         match s.parse().map_err(|_| "expected int") {
                             Ok(i) if i >= 1 => Ok(i),
                             Err(e) => Err(e),
                             _ => Err("count must be at least 1"),
                         }
                     })
                     .map(|i| Action::SetFreeColorCount(i))
            }
            // "fixedcolors" => words.next(),
            _ => Err("action not recognized"),
        }
    })
}


fn line_to_target(line: &str) -> Result<Target, &str> {
    let mut line = line.trim().split(" ");
    let direction = try!(line.next().ok_or("expected string").and_then(|string| {
        match string {
            "minimize" => Ok(Minimize),
            "maximize" => Ok(Maximize),
            "approximate" => {
                line.next()
                    .ok_or("expected string")
                    .and_then(|s| s.parse().map_err(|_| "expected float"))
                    .map(|f| Approximate(f))
            }

            _ => Err("expected minimize, maximize or approximate"),
        }
    }));

    let stat = try!(line.next().ok_or("expected string").and_then(|string| {
        match string {
            "mean" => Ok(Mean),
            "stddev" => Ok(StdDev),
            "min" => Ok(Min),
            "max" => Ok(Max),
            _ => Err("expected mean, stddev, min or max"),
        }
    }));

    let parameter = try!(line.next().ok_or("expected string").and_then(|string| {
        match string {
            "chroma" => Ok(Chroma),
            "luminance" => Ok(Luminance),
            "freedist" => Ok(FreeDistance),
            "fixeddist" => Ok(FixedDistance),
            _ => Err("expected chroma, luminance, freedist or fixeddist"),
        }
    }));

    let factor = line.next().and_then(|s| s.parse().ok()).unwrap_or(1.0);

    let exponent = line.next().and_then(|s| s.parse().ok()).unwrap_or(1);

    Ok(Target::new(direction,
                   stat,
                   parameter,
                   Strength {
                       factor: factor,
                       exponent: exponent,
                   }))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_set_count() {
        let line = "fg_color_count 6";
        assert_eq!(line_to_action(line).unwrap(), Action::SetFreeColorCount(6));
    }

    #[test]
    fn test_set_fg_colors() {
        let line = "preset_fg_colors rgb(255, 118, 240) rgb(173, 233, 255)";
        assert_eq!(line_to_action(line).unwrap(),
                   Action::SetPresetColors(vec![Lab::new(0.0, 0.0, 0.0), Lab::new(0.0, 0.0, 0.0)]));
    }

    #[test]
    fn test_set_bg_colors() {
        let line = "bg_colors rgb(255, 255, 255) rgb(51, 51, 51)";
        assert_eq!(line_to_action(line).unwrap(),
                   Action::SetFixedColors(vec![Lab::new(0.0, 0.0, 0.0), Lab::new(0.0, 0.0, 0.0)]));
    }

    #[test]
    fn test_set_color_filters() {
        let line = "color_filters redshift deuter";
        assert_eq!(line_to_action(line).unwrap(),
                   Action::SetColorFilters(vec![Redshift, Deuter]));
    }
}
