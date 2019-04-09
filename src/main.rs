extern crate csv;
extern crate xml;

use std::io;
use std::str;
use regex::Regex;
use csv::ReaderBuilder;
use xml::writer::XmlEvent;
use xml::writer::EmitterConfig;

macro_rules! get_field {
  ($iter:expr, $str:expr) => ({
    let row = $iter.next().unwrap().expect("a CSV row");
    assert_eq!(row.get(0), Some($str));
    assert_eq!(row.get(1), Some(""));
    &row.get(2).unwrap().to_owned()
  })
}
macro_rules! append_row4 {
  ($iter:expr, $vec:expr, $str:expr) => ({
    let row = $iter.next().unwrap().expect("a CSV row");
    assert_eq!(row.get(0), Some(""));
    assert_eq!(row.get(1), Some($str));
    assert_eq!(row.get(2), Some(""));
    assert_eq!(row.get(3), Some(""));
    for i in 4..row.len() {
        $vec.push(row.get(i).unwrap().to_owned());
    }
  })
}
macro_rules! append_row5 {
  ($iter:expr, $vec:expr, $str0:expr, $str1:expr, $str2:expr, $str3:expr, $str4:expr) => ({
    let row = $iter.next().unwrap().expect("a CSV row");
    assert_eq!(row.get(0), Some($str0));
    assert_eq!(row.get(1), Some($str1));
    assert_eq!(row.get(2), Some($str2));
    assert_eq!(row.get(3), Some($str3));
    assert_eq!(row.get(4), Some($str4));
    for i in 5..row.len() {
        $vec.push(row.get(i).unwrap().to_owned());
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
  ($w:expr, $str0:expr, $str1:expr, $item:expr) => (write_str_att!($w, $str0, "ReasonCode", $str1, $item))
}
macro_rules! write_str {
  ($w:expr, $str0:expr, $item:expr) => ({
    $w.write(XmlEvent::start_element($str0)).unwrap();
    let numstr: &str = &$item;
    $w.write(numstr).unwrap();
    $w.write(XmlEvent::end_element()).unwrap();
  })
}

fn main() {

    let mut rdr = ReaderBuilder::new()
        .delimiter(b';')
        .flexible(true)
        .has_headers(false)
        .from_reader(io::stdin());

    let mut iter = rdr.records().peekable();

    let election_identifier_name: &str = get_field!(iter, "Verkiezing");// "Provinciale Staten Fryslân 2019";
    let election_date: &str = get_field!(iter, "Datum");// "2019-03-20"
    let gm_naam: &str = get_field!(iter, "Gebied"); // "Gemeente Tytsjerksteradiel"
    let gm_naam_sub: &str = {
        if gm_naam.len() > 10 && &gm_naam[..9] == "Gemeente " {
            &gm_naam[9..] // "Tytsjerksteradiel"
        } else {
            &gm_naam
        }
    };
    let mut gm_code: String = get_field!(iter, "Nummer").to_string(); // "0737"
    for _i in 0..4-gm_code.len() {
        gm_code.insert_str(0, "0");
    }
    let election_category: &str; // PS | AB
    let election_subcategory: &str;// PS1 | PS2 | AB1 | ..
    let election_domain: &str; // Fryslân
    let re = Regex::new(r"[^A-Za-zâ]").unwrap();

    if election_identifier_name.len() > 19 && &election_identifier_name[..19] == "Provinciale Staten " {
        election_category = "PS";
        election_domain = &election_identifier_name[19..election_identifier_name.len()-5];
        election_subcategory = { if
            election_domain == "Gelderland" ||
            election_domain == "Noord-Holland" ||
            election_domain == "Zuid-Holland" ||
            election_domain == "Noord-Brabant" ||
            election_domain == "Limburg" {
                "PS2" // Provincie got 2 election districts (kieskringen)
            } else {
                "PS1" // Provincie with just 1 election district (kieskring)
            }
        };
    } else if election_identifier_name.len() > 36 && &election_identifier_name[..36] == "Algemeen bestuur van het waterschap " {
        election_category = "AB";
        election_domain = &election_identifier_name[36..election_identifier_name.len()-5];
        election_subcategory = { if
            election_domain == "Noorderzijlvest" ||
            election_domain == "Fryslân" ||
            election_domain == "Hunze en Aa's" ||
            election_domain == "Zuiderzeeland" {
                "AB1" // Small water board council (less than 19 seats)
            } else {
                "AB2" // Large water board council
            }
        };
    } else {
        panic!("not yet implemented");
    }
    let election_identifier_id = { // PS2019_Fryslan
        let ascii = election_domain.replace("â","a");
        let normalized =  re.replace_all(&ascii, "");
        &format!("{}{}_{}", &election_category, &election_date[0..4], &normalized)
    };

    let creation_date_time = &format!("{}T12:34:56.789", &election_date);//"2019-03-21T04:04:58.347";//"2019-03-21T06:57:22.834";

    let mut naam: Vec<String> = vec![];
    let mut gebiednummer: Vec<String> = vec![];
    let mut postcode: Vec<String> = vec![];
    let mut opgeroepenen: Vec<String> = vec![];
    let mut geldige_stempas: Vec<String> = vec![];
    let mut geldig_volmachtbewijs: Vec<String> = vec![];
    let mut geldige_kiezerspas: Vec<String> = vec![];
    let mut toegelaten_kiezers: Vec<String> = vec![];
    let mut geldige_stembiljetten: Vec<String> = vec![];
    let mut blanco_stembiljetten: Vec<String> = vec![];
    let mut ongeldige_stembiljetten: Vec<String> = vec![];
    let mut aangetroffen_stembiljetten: Vec<String> = vec![];
    let mut meer_stembiljetten_dan_toegelaten_kiezers: Vec<String> = vec![];
    let mut minder_stembiljetten_dan_toegelaten_kiezers: Vec<String> = vec![];
    let mut kiezers_met_stembiljet_hebben_niet_gestemd: Vec<String> = vec![];
    let mut er_zijn_te_weinig_stembiljetten_uitgereikt: Vec<String> = vec![];
    let mut er_zijn_te_veel_stembiljetten_uitgereikt: Vec<String> = vec![];
    let mut geen_verklaring: Vec<String> = vec![];
    let mut andere_verklaring: Vec<String> = vec![];
    let mut lijstnummer: Vec<String> = vec![];
    let mut partij: Vec<String> = vec![];
    let mut kandidaatnummer: Vec<String> = vec![];
    let mut kandidaatnaam: Vec<String> = vec![];
    let mut kandidaatstemmen: Vec<Vec<String>> = vec![];

    if match iter.peek() { Some(Ok(c)) => c.get(0), _ => None } == Some("") {
        iter.next().unwrap().expect("a CSV row");
    }
    append_row5!(iter, naam, "Lijstnummer", "Aanduiding", "Volgnummer", "Naam kandidaat", "Totaal");
    append_row5!(iter, gebiednummer, "Gebiednummer", "", "", "", "");
    if match iter.peek() { Some(Ok(c)) => c.get(0), _ => None } == Some("Postcode") {
        append_row5!(iter, postcode, "Postcode", "", "", "", "");
    } else {
        for _i in 0..gebiednummer.len() {
            postcode.push("".to_string());
        }
    }

    append_row4!(iter, opgeroepenen, "opgeroepenen");
    append_row4!(iter, geldige_stempas, "geldige stempas");
    append_row4!(iter, geldig_volmachtbewijs, "geldig volmachtbewijs");
    append_row4!(iter, geldige_kiezerspas, "geldige kiezerspas");
    append_row4!(iter, toegelaten_kiezers, "toegelaten kiezers");
    append_row4!(iter, geldige_stembiljetten, "geldige stembiljetten");
    append_row4!(iter, blanco_stembiljetten, "blanco stembiljetten");
    append_row4!(iter, ongeldige_stembiljetten, "ongeldige stembiljetten");
    append_row4!(iter, aangetroffen_stembiljetten, "aangetroffen stembiljetten");
    append_row4!(iter, meer_stembiljetten_dan_toegelaten_kiezers, "meer stembiljetten dan toegelaten kiezers ");
    append_row4!(iter, minder_stembiljetten_dan_toegelaten_kiezers, "minder stembiljetten dan toegelaten kiezers ");
    append_row4!(iter, kiezers_met_stembiljet_hebben_niet_gestemd, "kiezers met stembiljet hebben niet gestemd");
    append_row4!(iter, er_zijn_te_weinig_stembiljetten_uitgereikt, "er zijn te weinig stembiljetten uitgereikt");
    append_row4!(iter, er_zijn_te_veel_stembiljetten_uitgereikt, "er zijn te veel stembiljetten uitgereikt");
    append_row4!(iter, geen_verklaring, "geen verklaring");
    append_row4!(iter, andere_verklaring, "andere verklaring");

    for _i in 0..opgeroepenen.len() {
        kandidaatstemmen.push(vec![]);
    }
    for result in iter {
        let row = result.expect("a CSV row");
        lijstnummer.push(row.get(0).unwrap().to_owned());
        partij.push(row.get(1).unwrap().to_owned());
        kandidaatnummer.push(row.get(2).unwrap().to_owned());
        kandidaatnaam.push(row.get(3).unwrap().to_owned());
        for i in 4..row.len() {
            kandidaatstemmen[i-4].push(row.get(i).unwrap().to_owned());
        }
    }

    let mut b = Vec::new();
    let mut w = EmitterConfig::new().write_document_declaration(false).autopad_comments(false).create_writer(&mut b);

    w.write(XmlEvent::start_element("EML")
        .default_ns("urn:oasis:names:tc:evs:schema:eml")
        .ns("ds", "http://www.w3.org/2000/09/xmldsig#")
        .ns("kr", "http://www.kiesraad.nl/extensions")
        .ns("rg", "http://www.kiesraad.nl/reportgenerator")
        .ns("xal", "urn:oasis:names:tc:ciq:xsdschema:xAL:2.0")
        .ns("xnl", "urn:oasis:names:tc:ciq:xsdschema:xNL:2.0")
        .ns("xsi", "http://www.w3.org/2001/XMLSchema-instance")
        .attr("Id", "510b")
        .attr("SchemaVersion", "5")
        .attr("xsi:schemaLocation", "urn:oasis:names:tc:evs:schema:eml 510-count-v5-0.xsd http://www.kiesraad.nl/extensions kiesraad-eml-extensions.xsd")
    ).unwrap();
        w.write(XmlEvent::comment("Created by: Ondersteunende Software Verkiezingen by IVU Traffic Technologies AG, program: P4_PSB, version: 2.23.6")).unwrap();

        write_str!(w, "TransactionId", "1");

        w.write(XmlEvent::start_element("ManagingAuthority")).unwrap();
            write_str_att!(w, "AuthorityIdentifier", "Id", &gm_code, gm_naam_sub);
            write_str!(w, "AuthorityAddress", "");
        w.write(XmlEvent::end_element()).unwrap(); //ManagingAuthority

        write_str!(w, "kr:CreationDateTime", creation_date_time);

        write_str_att!(w, "ds:CanonicalizationMethod", "Algorithm", "http://www.w3.org/TR/2001/REC-xml-c14n-20010315#WithComments", "");

        w.write(XmlEvent::start_element("Count")).unwrap();

            write_str!(w, "EventIdentifier", "");

            w.write(XmlEvent::start_element("Election")).unwrap();
                w.write(XmlEvent::start_element("ElectionIdentifier").attr("Id", election_identifier_id)).unwrap();

                    write_str!(w, "ElectionName", election_identifier_name);
                    write_str!(w, "ElectionCategory", election_category);
                    write_str!(w, "kr:ElectionSubcategory", election_subcategory);
                    write_str!(w, "kr:ElectionDomain", election_domain);
                    write_str!(w, "kr:ElectionDate", election_date);

                w.write(XmlEvent::end_element()).unwrap(); //ElectionIdentifier

                w.write(XmlEvent::start_element("Contests")).unwrap();

                    w.write(XmlEvent::start_element("Contest")).unwrap();

                        write_str_att!(w, "ContestIdentifier", "Id", "geen", "");

                        for sb in 0..kandidaatstemmen.len() {
                            if sb == 0 {
                                w.write(XmlEvent::start_element("TotalVotes")).unwrap();
                            } else {
                                w.write(XmlEvent::start_element("ReportingUnitVotes")).unwrap();
                                write_str_att!(w,
                                        "ReportingUnitIdentifier",
                                        "Id",
                                        &format!("{}::SB{}", &gm_code, &gebiednummer[sb - 1]),
                                        format!("Stembureau {}{}", &naam[sb - 1], {
                                            if &postcode[sb - 1] != "" { format!(" (postcode: {})", &postcode[sb - 1]) }
                                            else {"".to_string()}
                                        })
                                );
                            }

                            for i in 0..lijstnummer.len() {
                                w.write(XmlEvent::start_element("Selection")).unwrap();

                                let nr = &lijstnummer[i];
                                if nr != "" {
                                    w.write(XmlEvent::start_element("AffiliationIdentifier").attr("Id", nr)).unwrap();
                                        write_str!(w, "RegisteredName", partij[i]);
                                    w.write(XmlEvent::end_element()).unwrap(); //AffiliationIdentifier
                                } else {
                                    w.write(XmlEvent::start_element("Candidate")).unwrap();
                                        write_str_att!(w, "CandidateIdentifier", "Id", &kandidaatnummer[i], "");
                                    w.write(XmlEvent::end_element()).unwrap(); //Candidate
                                }
                                    write_str!(w, "ValidVotes", kandidaatstemmen[sb][i]);
                                w.write(XmlEvent::end_element()).unwrap(); //Selection
                            }
                            write_str!(w, "Cast", opgeroepenen[sb]);
                            write_str!(w, "TotalCounted", geldige_stembiljetten[sb]);
                            write_reason!(w, "RejectedVotes", "ongeldig", ongeldige_stembiljetten[sb]);
                            write_reason!(w, "RejectedVotes", "blanco", blanco_stembiljetten[sb]);
                            write_reason!(w, "UncountedVotes", "geldige stempassen", geldige_stempas[sb]);
                            write_reason!(w, "UncountedVotes", "geldige volmachtbewijzen", geldig_volmachtbewijs[sb]);
                            write_reason!(w, "UncountedVotes", "geldige kiezerspassen", geldige_kiezerspas[sb]);
                            write_reason!(w, "UncountedVotes", "toegelaten kiezers", toegelaten_kiezers[sb]);
                            write_reason!(w, "UncountedVotes", "meer getelde stembiljetten", meer_stembiljetten_dan_toegelaten_kiezers[sb]);
                            write_reason!(w, "UncountedVotes", "minder getelde stembiljetten", minder_stembiljetten_dan_toegelaten_kiezers[sb]);
                            write_reason!(w, "UncountedVotes", "meegenomen stembiljetten", kiezers_met_stembiljet_hebben_niet_gestemd[sb]);
                            write_reason!(w, "UncountedVotes", "te weinig uitgereikte stembiljetten", er_zijn_te_weinig_stembiljetten_uitgereikt[sb]);
                            write_reason!(w, "UncountedVotes", "te veel uitgereikte stembiljetten", er_zijn_te_veel_stembiljetten_uitgereikt[sb]);
                            write_reason!(w, "UncountedVotes", "geen verklaring", geen_verklaring[sb]);
                            write_reason!(w, "UncountedVotes", "andere verklaring", andere_verklaring[sb]);

                            w.write(XmlEvent::end_element()).unwrap(); //TotalVotes or ReportingUnitVotes
                        }

                    w.write(XmlEvent::end_element()).unwrap(); //Contest
                w.write(XmlEvent::end_element()).unwrap(); //Contests
            w.write(XmlEvent::end_element()).unwrap(); //Election
        w.write(XmlEvent::end_element()).unwrap(); //Count
    w.write(XmlEvent::end_element()).unwrap(); //EML
    print!("{}", str::from_utf8(&b).unwrap());
}
