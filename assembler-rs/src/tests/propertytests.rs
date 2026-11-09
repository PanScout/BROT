use isa::ImmVal::{Full, Long, Short};
use proptest::prelude::*;

use isa::{ImmTypeLabel::*, ImmVal};
use crate::*;
use crate::string_handling::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn tokenize_rejects_garbage(s in "[A-Z0-9R ,#-]{0,100}")
    {
        let ret = tokenize_instruction(&s);
        prop_assert!(ret.is_err());
    }

    // Whatever comes out is clean
    #[test]
    fn clean_source_invariants(src in "[A-Za-z0-9 ,;/\n\t]{0,200}") {
        if let Ok(lines) = clean_input(&src) {
            let total = src.lines().count();
            for w in lines.windows(2) {
                prop_assert!(w[0].1 < w[1].1, "line numbers not increasing");
            }
            for (s, n) in &lines {
                prop_assert!(!s.is_empty());
                prop_assert!(!s.contains("//"));
                prop_assert_eq!(s.trim(), s.as_str());
                prop_assert!(*n >= 1 && *n <= total);
            }
        }
    }

    #[test]
    fn trve_gibberish_in_input(src in r"\PC*")
    {
        let _ = clean_input(&src);
    }


    #[test]
    fn trve_gibberish_in_imm(
    src in r"\PC*",
    imm_type in prop::sample::select(vec![
        UINT, UMOD, SINT, SMOD, Q15_24, Q7_32, Q23_16, Q31_8, UPACKED, SPACKED, BOOL, ]),)
        {
        let _ = process_imm(&src, imm_type);
        }

        #[test]
        fn test_fixed_point(
            fp in r"-?[0-9]{1,3}\.[0-9]{1,10}",
            (imm_type, frac) in prop::sample::select(vec![(Q31_8, 8), (Q23_16, 16), (Q15_24, 24), (Q7_32, 32)]),
        ) {
            // Sign-extend whatever slot came back, like the CPU does
            fn signed(imm: &ImmVal) -> i64 {
                let (raw, bits) = match imm {
                    Short(v) => (v.value() as u64, 18),
                    Long(v) => (v.value() as u64, 23),
                    Full(v) => (v.value(), 40),
                };
                ((raw << (64 - bits)) as i64) >> (64 - bits)
            }

            let x: f64 = fp.parse().unwrap();
            let k = (x * 2f64.powi(frac)).round() as i64;
            let result = process_imm(&fp, imm_type);

            if (-(1i64 << 39)..(1i64 << 39)).contains(&k) {
                prop_assert_eq!(signed(&result.unwrap()), k, "{} with {} frac bits", fp, frac);
            } else {
                prop_assert!(result.is_err(), "accepted out-of-range {}", fp);
            }
        }



    // Adding comments changes nothing
    #[test]
    fn comments_ignored(src in "[A-Z0-9 ,;\n]{0,300}", comment in "[ -~]{0,100}") {
    let commented: String = src.lines()
        .map(|l| format!("{l} //{comment}"))
        .collect::<Vec<_>>()
        .join("\n");
        prop_assert_eq!(clean_input(&commented), clean_input(&src));
    }
}
