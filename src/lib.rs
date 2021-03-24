extern crate chrono;
extern crate csv;
extern crate regex;
extern crate xml;
//#![feature(collections)]
//#![feature(iterator_step_by)]
extern crate crypto;

use self::crypto::digest::Digest;
use self::crypto::sha2::Sha256;
use chrono::NaiveDate;
use csv::ReaderBuilder;
use regex::Regex;
use std::io::{Read, Write};
use std::str;
use xml::writer::{EmitterConfig, EventWriter, XmlEvent};

macro_rules! get_field {
  ($iter:expr, $str:expr) => ({
    let row = $iter.next().unwrap().expect("a CSV row");
    assert_eq!(row.get(0), Some($str));
    assert_eq!(row.get(1), Some(""));
    row.get(2).unwrap().to_owned().to_string()
  })
}
macro_rules! append_row4u32 {
  ($iter:expr, $obj:ident, $key:ident, $str:expr) => ({
    let row = $iter.next().unwrap().expect("a CSV row");
    assert_eq!(row.get(0), Some(""));
    assert_eq!(row.get(1), Some($str));
    assert_eq!(row.get(2), Some(""));
    assert_eq!(row.get(3), Some(""));
    $obj.total_count.$key = row.get(4).unwrap().to_owned().parse::<u32>().expect("A positive number");
    for i in 5..row.len() {
        $obj.report_unit[i-5].count.$key = row.get(i).unwrap().to_owned().parse::<u32>().expect("A positive number");
    }
  })
}
macro_rules! append_row5somestr {
  ($iter:expr, $vec:expr, $key:ident, $str:expr) => ({
    let row = $iter.next().unwrap().expect("a CSV row");
    assert_eq!(row.get(0), Some($str));
    assert_eq!(row.get(1), Some(""));
    assert_eq!(row.get(2), Some(""));
    assert_eq!(row.get(3), Some(""));
    assert_eq!(row.get(4), Some(""));
    for i in 5..row.len() {
         $vec[i-5].$key = Some(row.get(i).unwrap().to_owned());
    }
  })
}
macro_rules! append_row5u32 {
  ($iter:expr, $vec:expr, $key:ident, $str:expr) => ({
    let row = $iter.next().unwrap().expect("a CSV row");
    assert_eq!(row.get(0), Some($str));
    assert_eq!(row.get(1), Some(""));
    assert_eq!(row.get(2), Some(""));
    assert_eq!(row.get(3), Some(""));
    assert_eq!(row.get(4), Some(""));
    for i in 5..row.len() {
        $vec[i-5].$key = row.get(i).unwrap().to_owned().parse::<u32>().expect("A positive number");
    }
  })
}
macro_rules! write_str_att {
  ($w:expr, $str0:expr, $str1:expr, $str2:expr, $item:expr) => ({
    $w.write(XmlEvent::start_element($str0).attr($str1, $str2)).unwrap();
    let numstr: &str = &$item;
    $w.write(numstr).unwrap();
    $w.write(XmlEvent::end_element()).unwrap();
  })
}
macro_rules! write_reason {
  ($w:expr, $str0:expr, $str1:expr, $item:expr) => (write_str_att!($w, $str0, "ReasonCode", $str1, $item.to_string()))
}
macro_rules! write_str {
  ($w:expr, $str0:expr, $item:expr) => ({
    $w.write(XmlEvent::start_element($str0)).unwrap();
    let numstr: &str = &$item;
    $w.write(numstr).unwrap();
    $w.write(XmlEvent::end_element()).unwrap();
  })
}

pub struct Contest {
    election_identifier_name: String,// e.g. "Provinciale Staten Fryslân 2019";
    election_date: String,// e.g. "2019-03-20"
    area_name: String,// e.g. "Gemeente Tytsjerksteradiel"
    area_name_short: String,// e.g. "Tytsjerksteradiel"
    area_code: String,// e.g. "0737"
    election_category: String, // e.g. PS | AB
    election_subcategory: String,// e.g. PS1 | PS2 | AB1 | ..
    election_domain: Option<String>, // e.g. "Fryslân"
    election_identifier_id: String, // e.g. "PS2019_Fryslan"
    creation_date_time: String,// YYYY-MM-DDTHH:mm:ss.iii format
    contest_id: String,// geen | election district id (kieskring)
    contest_name: String,// "" | election district name (kieskring)
    total_count: Count,
    report_unit: Vec<ReportUnit>,
    lists: Vec<List>
}

pub struct List {
    number: u8,
    name: String,
    candidates: Vec<Candidate>
}

pub struct Candidate {
    number: u8,
    name: String
}

pub struct ReportUnit {
    name: String,
    area_number: u32,
    zipcode: Option<String>,
    count: Count
}

pub struct Count {
    cast: u32,
    geldige_stempas: u32,
    geldig_volmachtbewijs: u32,
    geldige_kiezerspas: u32,
    toegelaten_kiezers: u32,
    toegelaten_kiezers_briefstembureaus_gemeente: u32,
    geldige_stembiljetten: u32,
    blanco_stembiljetten: u32,
    ongeldige_stembiljetten: u32,
    aangetroffen_stembiljetten: u32,
    meer_stembiljetten_dan_toegelaten_kiezers: u32,
    minder_stembiljetten_dan_toegelaten_kiezers: u32,
    kiezers_met_stembiljet_hebben_niet_gestemd: u32,
    er_zijn_te_weinig_stembiljetten_uitgereikt: u32,
    er_zijn_te_veel_stembiljetten_uitgereikt: u32,
    geen_stembiljet_in_enveloppe_briefstembureaus: u32,
    meer_stembiljetten_in_een_enveloppe_briefstembureaus: u32,
    geen_verklaring: u32,
    andere_verklaring: u32,
    votes: Vec<Vec<u32>>
}

fn difference(s1: &str, s2: &str) -> u8 {
    let mut diff = 0;

    for (c1, c2) in s1.chars().zip(s2.chars()) {
        if c1 != c2 {
            diff+=1;
        }
    }

    diff
}

pub fn contest2eml_find_sha (contest: &Contest, hashcode: String, optversions: Vec<String>, optdates: Option<String>) -> String {
    let eml = contest2eml(contest);
    {
        let mut parts = eml.split("version: 2.24.3");
        let first = parts.next().unwrap();
        let last: String = parts.next().unwrap().to_string();
        let date = format!("{}T12:34:56.789", &contest.election_date);
        let mut parts2 = last.split(&date);
        let middle = parts2.next().unwrap();
        let last = parts2.next().unwrap();

        let versions = optversions;
        let mut dates = vec![];
        dates.push(optdates.unwrap());

        //let versions = ["version: 2.24.3"];
        //let dates = ["2019-05-23T23:"];

        'search: for version in versions.iter() {
            // for date in dates.iter() {//13:51:42.081
            //     for hour in 22..23 { //0..23 {
                     for minute in 39..40 { //0..59 {
                         for second in 48..50 { //0..59 {
                            for millisecond in 0..1000 { //0..999 {
                                let mut hasher = Sha256::new();
                                hasher.input_str(&first as &str);
                                hasher.input_str(&versions[0] as &str);
                                hasher.input_str(&middle as &str);
                                // hasher.input_str(&format!("{}T{:02}:{:02}:{:02}.{:03}", date, hour, minute, second, millisecond) as &str);
                                // println!("{:}",&format!("{}T{:02}:{:02}:{:02}.{:03}", date, hour, minute, second, millisecond) as &str );
                                println!("{}{:02}:{:02}.{:03}", dates[0], minute, second, millisecond);
                                hasher.input_str(&format!("{}{:02}:{:02}.{:03}", dates[0], minute, second, millisecond) as &str);
                                hasher.input_str(&last as &str);
                                let hex = hasher.result_str();
                                println!("{}",hex);
                                let d = difference(&hex, &hashcode);
                                if hex == hashcode || d < 20 {
                                    println!("DONE");
                                    println!("found {}", &hex);
                                    if d > 0 {
                                        println!("searched for {} of by {} chars", &hashcode, d);
                                    }
                                    // print!("{}",&first as &str);
                                    // print!("{}",&version as &str);
                                    // print!("{}",&middle as &str);
                                    // //print!("{}",&format!("{}T{:02}:{:02}:{:02}.{:03}", date, hour, minute, second, millisecond) as &str);
                                    // print!("{}",&format!("{}.{:03}", dates[0], millisecond) as &str);
                                    // print!("{}",&last as &str);
                                    break 'search;
                                }
                            }
                         }
                    }
            //     }
            // }
        }
    }
    eml
}

pub fn contest2eml (contest: &Contest) -> String {

    let mut b = Vec::new();
    {
        let mut w = EmitterConfig::new().write_document_declaration(false).autopad_comments(false).create_writer(&mut b);

        w.write(XmlEvent::start_element("EML")
            .default_ns("urn:oasis:names:tc:evs:schema:eml")
            .ns("ds", "http://www.w3.org/2000/09/xmldsig#")
            .ns("kr", "http://www.kiesraad.nl/extensions")
            //.ns("rg", "http://www.kiesraad.nl/reportgenerator")
            .ns("xal", "urn:oasis:names:tc:ciq:xsdschema:xAL:2.0")
            .ns("xnl", "urn:oasis:names:tc:ciq:xsdschema:xNL:2.0")
            //.ns("xsi", "http://www.w3.org/2001/XMLSchema-instance")
            .attr("Id", "510b")
            .attr("SchemaVersion", "5")
            //.attr("xsi:schemaLocation", "urn:oasis:names:tc:evs:schema:eml 510-count-v5-0.xsd http://www.kiesraad.nl/extensions kiesraad-eml-extensions.xsd")
        ).unwrap();
            //w.write(XmlEvent::comment("Created by: Ondersteunende Software Verkiezingen by IVU Traffic Technologies AG, program: P4_PSB, version: 2.24.3")).unwrap();

            write_str!(w, "TransactionId", "1");

            w.write(XmlEvent::start_element("ManagingAuthority")).unwrap();
                write_str_att!(w, "AuthorityIdentifier", "Id", &contest.area_code, contest.area_name_short);
                write_str!(w, "AuthorityAddress", "");
            w.write(XmlEvent::end_element()).unwrap(); //ManagingAuthority

            write_str!(w, "kr:CreationDateTime", contest.creation_date_time);

            write_str_att!(w, "ds:CanonicalizationMethod", "Algorithm", "http://www.w3.org/TR/2001/REC-xml-c14n-20010315#WithComments", "");

            w.write(XmlEvent::start_element("Count")).unwrap();

                write_str!(w, "EventIdentifier", "");

                w.write(XmlEvent::start_element("Election")).unwrap();
                    w.write(XmlEvent::start_element("ElectionIdentifier").attr("Id", &contest.election_identifier_id)).unwrap();

                        write_str!(w, "ElectionName", contest.election_identifier_name);
                        write_str!(w, "ElectionCategory", contest.election_category);
                        write_str!(w, "kr:ElectionSubcategory", contest.election_subcategory);
                        if let Some(ref domain) = contest.election_domain {
                            write_str!(w, "kr:ElectionDomain", domain);
                        }
                        write_str!(w, "kr:ElectionDate", contest.election_date);

                    w.write(XmlEvent::end_element()).unwrap(); //ElectionIdentifier

                    w.write(XmlEvent::start_element("Contests")).unwrap();

                        w.write(XmlEvent::start_element("Contest")).unwrap();

                            w.write(XmlEvent::start_element("ContestIdentifier").attr("Id", &contest.contest_id)).unwrap();
                            if contest.contest_name.len() == 0 {
                                w.write("").unwrap();
                            } else {
                                write_str!(w, "ContestName", contest.contest_name);
                            }
                            w.write(XmlEvent::end_element()).unwrap(); //ContestIdentifier

                            w.write(XmlEvent::start_element("TotalVotes")).unwrap();
                            write_count(&mut w, &contest.lists, &contest.total_count);
                            w.write(XmlEvent::end_element()).unwrap(); //TotalVotes

                            for ru in &contest.report_unit {
                                w.write(XmlEvent::start_element("ReportingUnitVotes")).unwrap();
                                    write_str_att!(w,
                                        "ReportingUnitIdentifier",
                                        "Id",
                                        &format!("{}::SB{}", &contest.area_code, &ru.area_number),
                                        &format!("Stembureau {}{}", &ru.name, match &ru.zipcode {
                                            Some(ref zip) => format!(" (postcode: {})", &zip),
                                            None => "".to_string()
                                        })
                                    );
                                    write_count(&mut w, &contest.lists, &ru.count);
                                w.write(XmlEvent::end_element()).unwrap(); //ReportingUnitVotes
                            }
                        w.write(XmlEvent::end_element()).unwrap(); //Contest
                    w.write(XmlEvent::end_element()).unwrap(); //Contests
                w.write(XmlEvent::end_element()).unwrap(); //Election
            w.write(XmlEvent::end_element()).unwrap(); //Count
        w.write(XmlEvent::end_element()).unwrap(); //EML
    }
    str::from_utf8(&b).unwrap().to_string()
}

fn write_count<W: Write>(w: &mut EventWriter<W>, lists: &Vec<List>, c: &Count) {
    for i in 0..lists.len() {
        w.write(XmlEvent::start_element("Selection")).unwrap();
            w.write(XmlEvent::start_element("AffiliationIdentifier").attr("Id", &lists[i].number.to_string())).unwrap();
                write_str!(w, "RegisteredName", lists[i].name);
            w.write(XmlEvent::end_element()).unwrap(); //AffiliationIdentifier
            write_str!(w, "ValidVotes", c.votes[i][0].to_string());
        w.write(XmlEvent::end_element()).unwrap(); //Selection

        for j in 0..lists[i].candidates.len() {
            w.write(XmlEvent::start_element("Selection")).unwrap();
                w.write(XmlEvent::start_element("Candidate")).unwrap();
                    write_str_att!(w, "CandidateIdentifier", "Id", &lists[i].candidates[j].number.to_string(), "");
                w.write(XmlEvent::end_element()).unwrap(); //Candidate
                write_str!(w, "ValidVotes", c.votes[i][j + 1].to_string());
            w.write(XmlEvent::end_element()).unwrap(); //Selection
        }

    }
    write_str!(w, "Cast", c.cast.to_string());
    write_str!(w, "TotalCounted", c.geldige_stembiljetten.to_string());
    write_reason!(w, "RejectedVotes", "ongeldig", c.ongeldige_stembiljetten);
    write_reason!(w, "RejectedVotes", "blanco", c.blanco_stembiljetten);
    write_reason!(w, "UncountedVotes", "geldige stempassen", c.geldige_stempas);
    write_reason!(w, "UncountedVotes", "geldige volmachtbewijzen", c.geldig_volmachtbewijs);
    write_reason!(w, "UncountedVotes", "geldige kiezerspassen", c.geldige_kiezerspas);
    write_reason!(w, "UncountedVotes", "toegelaten kiezers", c.toegelaten_kiezers);
    write_reason!(w, "UncountedVotes", "toegelaten kiezers (Briefstembureaus gemeente)", c.toegelaten_kiezers_briefstembureaus_gemeente);
    write_reason!(w, "UncountedVotes", "meer getelde stembiljetten", c.meer_stembiljetten_dan_toegelaten_kiezers);
    write_reason!(w, "UncountedVotes", "minder getelde stembiljetten", c.minder_stembiljetten_dan_toegelaten_kiezers);
    write_reason!(w, "UncountedVotes", "meegenomen stembiljetten", c.kiezers_met_stembiljet_hebben_niet_gestemd);
    write_reason!(w, "UncountedVotes", "te weinig uitgereikte stembiljetten", c.er_zijn_te_weinig_stembiljetten_uitgereikt);
    write_reason!(w, "UncountedVotes", "te veel uitgereikte stembiljetten", c.er_zijn_te_veel_stembiljetten_uitgereikt);
    write_reason!(w, "UncountedVotes", "geen stembiljet in enveloppe (Briefstembureaus)", c.geen_stembiljet_in_enveloppe_briefstembureaus);
    write_reason!(w, "UncountedVotes", "meer stembiljetten in een enveloppe (Briefstembureaus)", c.meer_stembiljetten_in_een_enveloppe_briefstembureaus);
    write_reason!(w, "UncountedVotes", "geen verklaring", c.geen_verklaring);
    write_reason!(w, "UncountedVotes", "andere verklaring", c.andere_verklaring);
}

pub fn csv2contest(r: Box<Read>) -> Contest {
    let mut contest = Contest {
        election_identifier_name: "".to_string(),
        election_date: "".to_string(),
        area_name: "".to_string(),
        area_name_short: "".to_string(),
        area_code: "".to_string(),
        election_category: "".to_string(),
        election_subcategory: "".to_string(),
        election_domain: None,
        election_identifier_id: "".to_string(),
        creation_date_time: "".to_string(),
        contest_id: "geen".to_string(),
        contest_name: "".to_string(),
        total_count: Count {
            cast: 0,
            geldige_stempas: 0,
            geldig_volmachtbewijs: 0,
            geldige_kiezerspas: 0,
            toegelaten_kiezers: 0,
            toegelaten_kiezers_briefstembureaus_gemeente: 0,
            geldige_stembiljetten: 0,
            blanco_stembiljetten: 0,
            ongeldige_stembiljetten: 0,
            aangetroffen_stembiljetten: 0,
            meer_stembiljetten_dan_toegelaten_kiezers: 0,
            minder_stembiljetten_dan_toegelaten_kiezers: 0,
            kiezers_met_stembiljet_hebben_niet_gestemd: 0,
            er_zijn_te_weinig_stembiljetten_uitgereikt: 0,
            er_zijn_te_veel_stembiljetten_uitgereikt: 0,
            geen_stembiljet_in_enveloppe_briefstembureaus: 0,
            meer_stembiljetten_in_een_enveloppe_briefstembureaus: 0,
            geen_verklaring: 0,
            andere_verklaring: 0,
            votes: vec![]
        },
        report_unit: vec![],
        lists: vec![]
    };
    let mut rdr = ReaderBuilder::new()
        .delimiter(b';')
        .flexible(true)
        .has_headers(false)
        .from_reader(r);

    let mut iter = rdr.records().peekable();

    contest.election_identifier_name = get_field!(iter, "Verkiezing");
    contest.election_date = {
        let date = get_field!(iter, "Datum");
        if date.len() > 3 && &date[2..3] == "-" { // somebody reversed RFC3339 / ISO8601 order :(
            let d = NaiveDate::parse_from_str(&date, "%d-%m-%Y").expect("Datum is too corrupt");
            d.format("%Y-%m-%d").to_string()
        } else {
            date
        }
    };
    contest.area_name = get_field!(iter, "Gebied");
    contest.area_name_short = {
        if contest.area_name.len() > 10 && &contest.area_name[..9] == "Gemeente " {
            contest.area_name[9..].to_string()
        } else {
            contest.area_name.to_string()
        }
    };
    contest.area_code = get_field!(iter, "Nummer").to_string();
    for _i in 0..4-contest.area_code.len() {
        contest.area_code.insert_str(0, "0"); // Pad with zeros because of possible xlsx removal
    }
    let re = Regex::new(r"[^A-Za-zâ]").unwrap();

    if contest.election_identifier_name.len() > 19 && &contest.election_identifier_name[..19] == "Provinciale Staten " {
        contest.election_category = "PS".to_string();
        contest.election_domain = Some(contest.election_identifier_name[19..contest.election_identifier_name.len()-5].to_string());
        contest.election_subcategory = match contest.election_domain {
            Some(ref s) => match &s[..] {
                "Gelderland" |
                "Noord-Holland" |
                "Zuid-Holland" |
                "Noord-Brabant" |
                "Limburg" => "PS2".to_string(), // Provincie got 2 election districts (kieskringen)
                _ => "PS1".to_string() // Provincie with just 1 election district (kieskring)
            },
            None => panic!("The election_domain was expect to be some(thing)")
        };
        if &contest.election_subcategory == "PS2" {
            //panic!("not yet implemented content id fetch (xpath from verkiezingsdefinitie)");
        }
    } else if contest.election_identifier_name.len() > 25 && &contest.election_identifier_name[..25] == "Algemeen bestuur van het " {
        let offset = {
            if contest.election_identifier_name.len() > 36 && (&contest.election_identifier_name[25..36] == "waterschap " || &contest.election_identifier_name[25..36] == "wetterskip ") {
                36
            } else if contest.election_identifier_name.len() > 43 && &contest.election_identifier_name[25..43] == "hoogheemraadschap " {
                43
            } else {
                println!("{:?}", contest.election_identifier_name);
                panic!("not yet implemented water board variation");
            }
        };
        contest.election_category = "AB".to_string();
        contest.election_domain = Some(contest.election_identifier_name[offset..contest.election_identifier_name.len()-5].to_string());
        contest.election_subcategory = match contest.election_domain {
            Some(ref s) => match &s[..] {
                "Noorderzijlvest" |
                "Fryslân" |
                "Hunze en Aa's" |
                "Zuiderzeeland" => "AB1".to_string(), // Small water board council (less than 19 seats)
                _ =>  "AB2".to_string() // Large water board council
            },
            None => panic!("The election_domain was expect to be some(thing)")
        };
    } else if contest.election_identifier_name.len() > 19 && &contest.election_identifier_name[..19] == "Europees Parlement " {
        contest.election_category = "EP".to_string();
        contest.election_subcategory = "EP".to_string();
        contest.contest_id = "alle".to_string();
    } else if contest.election_identifier_name.len() > 30 && &contest.election_identifier_name[..33] == "Tweede Kamer der Staten-Generaal " {
        contest.election_category = "TK".to_string();
        contest.election_subcategory = "TK".to_string();
        contest.contest_id = "TODO".to_string();
    } else {
        panic!("not yet implemented election domain");
    }
    contest.election_identifier_id = {
        match contest.election_domain {
            Some(ref election_domain) => {
                let ascii = election_domain.replace("â","a");
                let normalized =  re.replace_all(&ascii, "");
                format!("{}{}_{}", &contest.election_category, &contest.election_date[0..4], &normalized).to_string()
            }
            None => format!("{}{}", &contest.election_category, &contest.election_date[0..4]).to_string()
        }
    };

    contest.creation_date_time = format!("{}T12:34:56.789", &contest.election_date);

    if match iter.peek() { Some(Ok(c)) => c.get(0), _ => None } == Some("") {
        iter.next().unwrap().expect("a CSV row");
    }
    {
        let row = iter.next().unwrap().expect("a CSV row");
        assert_eq!(row.get(0), Some("Lijstnummer"));
        assert_eq!(row.get(1), Some("Aanduiding"));
        assert_eq!(row.get(2), Some("Volgnummer"));
        assert_eq!(row.get(3), Some("Naam kandidaat"));
        assert_eq!(row.get(4), Some("Totaal"));
        for i in 5..row.len() {
            contest.report_unit.push(ReportUnit {
                name: row.get(i).unwrap().to_owned(),
                area_number: 0,
                zipcode: None,
                count: Count {
                    cast: 0,
                    geldige_stempas: 0,
                    geldig_volmachtbewijs: 0,
                    geldige_kiezerspas: 0,
                    toegelaten_kiezers: 0,
                    toegelaten_kiezers_briefstembureaus_gemeente: 0,
                    geldige_stembiljetten: 0,
                    blanco_stembiljetten: 0,
                    ongeldige_stembiljetten: 0,
                    aangetroffen_stembiljetten: 0,
                    meer_stembiljetten_dan_toegelaten_kiezers: 0,
                    minder_stembiljetten_dan_toegelaten_kiezers: 0,
                    kiezers_met_stembiljet_hebben_niet_gestemd: 0,
                    er_zijn_te_weinig_stembiljetten_uitgereikt: 0,
                    er_zijn_te_veel_stembiljetten_uitgereikt: 0,
                    geen_stembiljet_in_enveloppe_briefstembureaus: 0,
                    meer_stembiljetten_in_een_enveloppe_briefstembureaus: 0,
                    geen_verklaring: 0,
                    andere_verklaring: 0,
                    votes: vec![]
                }
            });
        }
    }

    append_row5u32!(iter, contest.report_unit, area_number, "Gebiednummer");
    if match iter.peek() { Some(Ok(c)) => c.get(0), _ => None } == Some("Postcode") {
        append_row5somestr!(iter, contest.report_unit, zipcode, "Postcode");
    }

    append_row4u32!(iter, contest, cast, "opgeroepenen");
    append_row4u32!(iter, contest, geldige_stempas, "geldige stempas");
    append_row4u32!(iter, contest, geldig_volmachtbewijs, "geldig volmachtbewijs");
    append_row4u32!(iter, contest, geldige_kiezerspas, "geldige kiezerspas");
    append_row4u32!(iter, contest, toegelaten_kiezers, "toegelaten kiezers");
    append_row4u32!(iter, contest, toegelaten_kiezers_briefstembureaus_gemeente, "toegelaten kiezers (Briefstembureaus gemeente)");
    append_row4u32!(iter, contest, geldige_stembiljetten, "geldige stembiljetten");
    append_row4u32!(iter, contest, blanco_stembiljetten, "blanco stembiljetten");
    append_row4u32!(iter, contest, ongeldige_stembiljetten, "ongeldige stembiljetten");
    append_row4u32!(iter, contest, aangetroffen_stembiljetten, "aangetroffen stembiljetten");
    append_row4u32!(iter, contest, meer_stembiljetten_dan_toegelaten_kiezers, "meer stembiljetten dan toegelaten kiezers");
    append_row4u32!(iter, contest, minder_stembiljetten_dan_toegelaten_kiezers, "minder stembiljetten dan toegelaten kiezers");
    append_row4u32!(iter, contest, kiezers_met_stembiljet_hebben_niet_gestemd, "kiezers met stembiljet hebben niet gestemd");
    append_row4u32!(iter, contest, er_zijn_te_weinig_stembiljetten_uitgereikt, "er zijn te weinig stembiljetten uitgereikt");
    append_row4u32!(iter, contest, er_zijn_te_veel_stembiljetten_uitgereikt, "er zijn te veel stembiljetten uitgereikt");
    append_row4u32!(iter, contest, geen_stembiljet_in_enveloppe_briefstembureaus, "geen stembiljet in enveloppe (Briefstembureaus)");
    append_row4u32!(iter, contest, meer_stembiljetten_in_een_enveloppe_briefstembureaus, "meer stembiljetten in een enveloppe (Briefstembureaus)");
    append_row4u32!(iter, contest, geen_verklaring, "geen verklaring");
    append_row4u32!(iter, contest, andere_verklaring, "andere verklaring");

    for result in iter {
        let row = result.expect("a CSV row");
        let first_column = row.get(0).unwrap().to_owned();
        if first_column.len() != 0 {
            contest.lists.push(List {
                number: first_column.parse::<u8>().expect("A number between 0 and 255"),
                name: row.get(1).unwrap().to_owned(),
                candidates: vec![]
            });
            contest.total_count.votes.push(vec![]);
            for i in 0..contest.report_unit.len() {
                contest.report_unit[i].count.votes.push(vec![]);
            }
            assert_eq!(row.get(2), Some(""));
            assert_eq!(row.get(3), Some(""));
        } else {
            assert_eq!(row.get(1), Some(""));
            contest.lists.last_mut().expect("A list before a candidate").candidates.push(Candidate {
                number: row.get(2).unwrap().to_owned().parse::<u8>().expect("A number between 0 and 255"),
                name: row.get(3).unwrap().to_owned()
            });
        }
        contest.total_count.votes.last_mut().expect("A list before a candidate").push(row.get(4).unwrap().to_owned().parse::<u32>().expect("A positive number"));
        for i in 5..row.len() {
            contest.report_unit[i-5].count.votes.last_mut().expect("A list before a candidate").push(row.get(i).unwrap().to_owned().parse::<u32>().expect("A positive number"));
        }
    }
    // Some CSV's are messed up in reporting unit order, so sort it here
    contest.report_unit.sort_by_key(|ru| ru.area_number);
    contest
}
