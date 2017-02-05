use std::borrow::ToOwned;
use std::collections::HashMap;
use regex::Regex;

/// A route `Expression` is a regular expression compiled from a
/// "template string" in which portions of a URL path are bound to
/// named groupings denoted by curly braces.
///
/// This expression can then be matched against the "Path" component
/// of a URI to extract that path into the series of named groupings.
#[derive(Clone)]
pub struct Expression {
	names: Vec<String>,
	regex: Regex,
}

impl Expression {

	/// `template` must take the following form: (pseudo-EBNF)
	///
	/// template = { "/" segment } ["/"] .
	/// segment  = "{" Name "}" | "{" Name ":" Pattern "}"
	///    - where `name` is a legal URI path segment
	///    - where `pattern` is an unanchored regular expression
	///
	pub fn from_template(template: &str) -> Result<Expression, String> {
		// temp variables
		let mut regex_string       = String::from("^"); // anchor to beginning of path
		let mut names: Vec<String> = Vec::new();

		let segments = match extract_segments(template) {
			Ok(segments) => { segments },
			Err(msg)     => { return Err(msg.to_owned()); },
		};

		for meta_segment in segments.iter() {
			let (ref preceding, ref segment) = *meta_segment;
			let tokens: Vec<String> = segment[..].split(':')
			                             .map(|x| { x.to_owned() })
			                             .collect();

			// TODO: do I really need to clone() here?
			let name = tokens[0].to_string();
			let patt = if tokens.len() == 2 {
				tokens[1].to_string()
			} else {
				String::from("[^/]*")
			};

			if &name[..] == "" || &patt[..] == "" {
				return Err(format!("missing name or pattern in: {}", template));
			}

			// TODO: Reverse regexp
			// TODO: Escape meta-characters in `name`
			names.push(name);
			regex_string.push_str(&format!("{}({})", preceding, patt)[..]);
		}

		// append the remaining bit of the path
		//
		// since we disallow nested braces; this is just the
		// suffix after the last closing brace.
		let trailing_chars = match template.rfind('}') {
			Some(last_brace_idx) => {
				&template[(last_brace_idx+1)..template.chars().count()]
			},
			None => {
				&template[0..template.chars().count()]
			},
		};

		regex_string.push_str(&format!("{}$", trailing_chars)[..]);
		debug!("generated route regex: {}", regex_string);

		Ok(Expression {
			names: names,
			regex: Regex::new(&regex_string[..]).unwrap(),
		})
	}

	pub fn is_match(&self, path: &str) -> bool {
		self.regex.is_match(path)
	}

	pub fn map_path(&self, path: &str) -> HashMap<String, String> {
		let mut results = HashMap::new();
		let captures    = self.regex.captures(path);

		// iterates over our list of named parameters and extracts
		// the corresponding capture group from the regex.
		//
		// the captures are offset by 1 because the first match
		// is the entire route.
		match captures {
			Some(captures) => {
				for idx in 0..self.names.len() {
					if let Some(binding) = captures.at(idx+1) {
						debug!("got route capture {}", binding);
						results.insert(self.names[idx].clone(), binding.to_owned());
					};
				}
			},
			None => {},
		};

		return results;
	}
}

/// Extract named parameters from a template string
///
/// Begins capturing a string at `{`, stops capturing a string
/// at `}`, fails if string capture contains `{` or `}` or if the
/// expression is unbalanced.
///
/// Returns an error describing the parsing failure, OR a
/// vector of matched segments.
fn extract_segments(input: &str) -> Result<Vec<(String,String)>, &'static str> {
	let mut input_buf = &input[..];

	// parser state
	let mut brace_count  =  0;
	let mut segment_text = String::new();
	let mut param_text   = String::new();

	// results
	let mut segments: Vec<(String,String)> = Vec::new();

	loop {
		// TODO(drbawb): chars() takes the first unicode scalar value
		// whereas the truncation of input-buf is bytewise.
		//
		// that being said this only affects the route-strings, not the user
		// supplied routes. so this should be fine until I use emoji in my route
		// definitions...
		//
		match input_buf.chars().nth(0) {
			Some(token) => {
				input_buf = &input_buf[1..]; // move slice forward 1 char.
				match token {
					'{' => { brace_count += 1; continue; },
					'}' => {
						brace_count -= 1;

						segments.push((segment_text.to_owned(),
						               param_text.to_owned()));
						param_text   = String::new();
						segment_text = String::new();
						continue;
					},
					_ if brace_count == 0 => { segment_text.push(token); continue; },
					_ if brace_count == 1 => { param_text.push(token); continue; },
					_ => { return Err("mismatched braces in route pattern"); },
				};
			},

			None => { break; },
		}
	}

	if brace_count == 0 {
		return Ok(segments);
	} else {
		return Err("missing closing brace in route pattern?");
	}

}

//
// tests
//

// test creation of a template
#[test]
fn test_build_template() {
	match Expression::from_template("/{foo}/{bar}") {
		Err(e) => { panic!("{}", e); },
		_      => {},
	}
}

#[test]
fn test_match_template() {
	let exp_1  = Expression::from_template("/{foo}/{bar}/baz").unwrap();
	let result = exp_1.map_path("/hello/world/baz");

	assert!("hello" == &result.get("foo").unwrap()[..]);
	assert!("world" == &result.get("bar").unwrap()[..]);
}


// test extracting named params from template
#[test]
fn test_template_extractor_count() {
	let pass_cases = [(1, "/foo/{bar}"), (2, "/{foo}/{bar}"), (1, "/foo/{bar}/baz")];
	let fail_cases = ["/{foo{bar}}/baz", "/{foo/bar", "/foo}/bar"];

	for test_case in pass_cases.iter() {
		let (expected_results, test_template) = *test_case;
		match extract_segments(test_template) {
			Ok(segment)  => {
				assert_eq!(expected_results, segment.len());
			},
			Err(e) => { panic!(e); },
		}
	}

	for test_case in fail_cases.iter() {
		match extract_segments(*test_case) {
			Ok(result) => { panic!("got {} unexpected results", result.len()); },
			Err(_)     => {},
		}
	}
}

#[test]
fn test_no_extractions() {
	// tests that a pattern w/ no extractable parameters
	// can still be matched...
	let template = Expression::from_template("/foo/bar").unwrap();

	assert!(template.is_match("/foo/bar") == true);
	assert!(template.is_match("/baz/qux") == false);
}

#[test]
fn test_template_extractor_values() {
	let template = "/foo/{bar:pat}";

	match extract_segments(template) {
		Ok(meta_segment) => {
			let (_, ref segment) = meta_segment[0];
			assert!("bar:pat" == &segment[..]);
		},
		Err(msg) => { panic!("Error while extracting segments {}", msg); },
	}
}
