extern crate csv;
extern crate xml;

use std::io;
use std::io::{Read};
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
    row.get(2).unwrap().to_owned().to_string()
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

struct OsvInternalData {
    election_identifier_name: String,// e.g. "Provinciale Staten Fryslân 2019";
    election_date: String,// e.g. "2019-03-20"
    gm_naam: String,// e.g. "Gemeente Tytsjerksteradiel"
    gm_naam_sub: String,// e.g. "Tytsjerksteradiel"
    gm_code: String,// e.g. "0737"
    election_category: String, // e.g. PS | AB
    election_subcategory: String,// e.g. PS1 | PS2 | AB1 | ..
    election_domain: String, // e.g. "Fryslân"
    election_identifier_id: String, // e.g. "PS2019_Fryslan"
    creation_date_time: String,// YYYY-MM-DDTHH:mm:ss.iii format
    naam: Vec<String>,
    gebiednummer: Vec<String>,
    postcode: Vec<String>,
    opgeroepenen: Vec<String>,
    geldige_stempas: Vec<String>,
    geldig_volmachtbewijs: Vec<String>,
    geldige_kiezerspas: Vec<String>,
    toegelaten_kiezers: Vec<String>,
    geldige_stembiljetten: Vec<String>,
    blanco_stembiljetten: Vec<String>,
    ongeldige_stembiljetten: Vec<String>,
    aangetroffen_stembiljetten: Vec<String>,
    meer_stembiljetten_dan_toegelaten_kiezers: Vec<String>,
    minder_stembiljetten_dan_toegelaten_kiezers: Vec<String>,
    kiezers_met_stembiljet_hebben_niet_gestemd: Vec<String>,
    er_zijn_te_weinig_stembiljetten_uitgereikt: Vec<String>,
    er_zijn_te_veel_stembiljetten_uitgereikt: Vec<String>,
    geen_verklaring: Vec<String>,
    andere_verklaring: Vec<String>,
    lijstnummer: Vec<String>,
    partij: Vec<String>,
    kandidaatnummer: Vec<String>,
    kandidaatnaam: Vec<String>,
    kandidaatstemmen: Vec<Vec<String>>
}

fn data2eml (data: OsvInternalData) -> String {

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
            write_str_att!(w, "AuthorityIdentifier", "Id", &data.gm_code, data.gm_naam_sub);
            write_str!(w, "AuthorityAddress", "");
        w.write(XmlEvent::end_element()).unwrap(); //ManagingAuthority

        write_str!(w, "kr:CreationDateTime", data.creation_date_time);

        write_str_att!(w, "ds:CanonicalizationMethod", "Algorithm", "http://www.w3.org/TR/2001/REC-xml-c14n-20010315#WithComments", "");

        w.write(XmlEvent::start_element("Count")).unwrap();

            write_str!(w, "EventIdentifier", "");

            w.write(XmlEvent::start_element("Election")).unwrap();
                w.write(XmlEvent::start_element("ElectionIdentifier").attr("Id", &data.election_identifier_id)).unwrap();

                    write_str!(w, "ElectionName", data.election_identifier_name);
                    write_str!(w, "ElectionCategory", data.election_category);
                    write_str!(w, "kr:ElectionSubcategory", data.election_subcategory);
                    write_str!(w, "kr:ElectionDomain", data.election_domain);
                    write_str!(w, "kr:ElectionDate", data.election_date);

                w.write(XmlEvent::end_element()).unwrap(); //ElectionIdentifier

                w.write(XmlEvent::start_element("Contests")).unwrap();

                    w.write(XmlEvent::start_element("Contest")).unwrap();

                        write_str_att!(w, "ContestIdentifier", "Id", "geen", "");

                        for sb in 0..data.kandidaatstemmen.len() {
                            if sb == 0 {
                                w.write(XmlEvent::start_element("TotalVotes")).unwrap();
                            } else {
                                w.write(XmlEvent::start_element("ReportingUnitVotes")).unwrap();
                                write_str_att!(w,
                                        "ReportingUnitIdentifier",
                                        "Id",
                                        &format!("{}::SB{}", &data.gm_code, &data.gebiednummer[sb - 1]),
                                        &format!("Stembureau {}{}", &data.naam[sb - 1], {
                                            if &data.postcode[sb - 1] != "" { format!(" (postcode: {})", &data.postcode[sb - 1]) }
                                            else {"".to_string()}
                                        })
                                );
                            }

                            for i in 0..data.lijstnummer.len() {
                                w.write(XmlEvent::start_element("Selection")).unwrap();

                                let nr = &data.lijstnummer[i];
                                if nr != "" {
                                    w.write(XmlEvent::start_element("AffiliationIdentifier").attr("Id", nr)).unwrap();
                                        write_str!(w, "RegisteredName", data.partij[i]);
                                    w.write(XmlEvent::end_element()).unwrap(); //AffiliationIdentifier
                                } else {
                                    w.write(XmlEvent::start_element("Candidate")).unwrap();
                                        write_str_att!(w, "CandidateIdentifier", "Id", &data.kandidaatnummer[i], "");
                                    w.write(XmlEvent::end_element()).unwrap(); //Candidate
                                }
                                    write_str!(w, "ValidVotes", data.kandidaatstemmen[sb][i]);
                                w.write(XmlEvent::end_element()).unwrap(); //Selection
                            }
                            write_str!(w, "Cast", data.opgeroepenen[sb]);
                            write_str!(w, "TotalCounted", data.geldige_stembiljetten[sb]);
                            write_reason!(w, "RejectedVotes", "ongeldig", data.ongeldige_stembiljetten[sb]);
                            write_reason!(w, "RejectedVotes", "blanco", data.blanco_stembiljetten[sb]);
                            write_reason!(w, "UncountedVotes", "geldige stempassen", data.geldige_stempas[sb]);
                            write_reason!(w, "UncountedVotes", "geldige volmachtbewijzen", data.geldig_volmachtbewijs[sb]);
                            write_reason!(w, "UncountedVotes", "geldige kiezerspassen", data.geldige_kiezerspas[sb]);
                            write_reason!(w, "UncountedVotes", "toegelaten kiezers", data.toegelaten_kiezers[sb]);
                            write_reason!(w, "UncountedVotes", "meer getelde stembiljetten", data.meer_stembiljetten_dan_toegelaten_kiezers[sb]);
                            write_reason!(w, "UncountedVotes", "minder getelde stembiljetten", data.minder_stembiljetten_dan_toegelaten_kiezers[sb]);
                            write_reason!(w, "UncountedVotes", "meegenomen stembiljetten", data.kiezers_met_stembiljet_hebben_niet_gestemd[sb]);
                            write_reason!(w, "UncountedVotes", "te weinig uitgereikte stembiljetten", data.er_zijn_te_weinig_stembiljetten_uitgereikt[sb]);
                            write_reason!(w, "UncountedVotes", "te veel uitgereikte stembiljetten", data.er_zijn_te_veel_stembiljetten_uitgereikt[sb]);
                            write_reason!(w, "UncountedVotes", "geen verklaring", data.geen_verklaring[sb]);
                            write_reason!(w, "UncountedVotes", "andere verklaring", data.andere_verklaring[sb]);

                            w.write(XmlEvent::end_element()).unwrap(); //TotalVotes or ReportingUnitVotes
                        }

                    w.write(XmlEvent::end_element()).unwrap(); //Contest
                w.write(XmlEvent::end_element()).unwrap(); //Contests
            w.write(XmlEvent::end_element()).unwrap(); //Election
        w.write(XmlEvent::end_element()).unwrap(); //Count
    w.write(XmlEvent::end_element()).unwrap(); //EML
    str::from_utf8(&b).unwrap().to_string()
}

fn csv2data(r: Box<Read>) -> OsvInternalData {
    let mut data = OsvInternalData {
        election_identifier_name: "".to_string(),
        election_date: "".to_string(),
        gm_naam: "".to_string(),
        gm_naam_sub: "".to_string(),
        gm_code: "".to_string(),
        election_category: "".to_string(),
        election_subcategory: "".to_string(),
        election_domain: "".to_string(),
        election_identifier_id: "".to_string(),
        creation_date_time: "".to_string(),
        naam: vec![],
        gebiednummer: vec![],
        postcode: vec![],
        opgeroepenen: vec![],
        geldige_stempas: vec![],
        geldig_volmachtbewijs: vec![],
        geldige_kiezerspas: vec![],
        toegelaten_kiezers: vec![],
        geldige_stembiljetten: vec![],
        blanco_stembiljetten: vec![],
        ongeldige_stembiljetten: vec![],
        aangetroffen_stembiljetten: vec![],
        meer_stembiljetten_dan_toegelaten_kiezers: vec![],
        minder_stembiljetten_dan_toegelaten_kiezers: vec![],
        kiezers_met_stembiljet_hebben_niet_gestemd: vec![],
        er_zijn_te_weinig_stembiljetten_uitgereikt: vec![],
        er_zijn_te_veel_stembiljetten_uitgereikt: vec![],
        geen_verklaring: vec![],
        andere_verklaring: vec![],
        lijstnummer: vec![],
        partij: vec![],
        kandidaatnummer: vec![],
        kandidaatnaam: vec![],
        kandidaatstemmen: vec![]
    };
    let mut rdr = ReaderBuilder::new()
        .delimiter(b';')
        .flexible(true)
        .has_headers(false)
        .from_reader(r);

    let mut iter = rdr.records().peekable();

    data.election_identifier_name = get_field!(iter, "Verkiezing");
    data.election_date = get_field!(iter, "Datum");
    data.gm_naam = get_field!(iter, "Gebied");
    data.gm_naam_sub = {
        if data.gm_naam.len() > 10 && &data.gm_naam[..9] == "Gemeente " {
            data.gm_naam[9..].to_string()
        } else {
            data.gm_naam.to_string()
        }
    };
    data.gm_code = get_field!(iter, "Nummer").to_string();
    for _i in 0..4-data.gm_code.len() {
        data.gm_code.insert_str(0, "0");
    }
    let re = Regex::new(r"[^A-Za-zâ]").unwrap();

    if data.election_identifier_name.len() > 19 && &data.election_identifier_name[..19] == "Provinciale Staten " {
        data.election_category = "PS".to_string();
        data.election_domain = data.election_identifier_name[19..data.election_identifier_name.len()-5].to_string();
        data.election_subcategory = { if
            data.election_domain == "Gelderland" ||
            data.election_domain == "Noord-Holland" ||
            data.election_domain == "Zuid-Holland" ||
            data.election_domain == "Noord-Brabant" ||
            data.election_domain == "Limburg" {
                "PS2".to_string() // Provincie got 2 election districts (kieskringen)
            } else {
                "PS1".to_string() // Provincie with just 1 election district (kieskring)
            }
        };
    } else if data.election_identifier_name.len() > 36 && &data.election_identifier_name[..36] == "Algemeen bestuur van het waterschap " {
        data.election_category = "AB".to_string();
        data.election_domain = data.election_identifier_name[36..data.election_identifier_name.len()-5].to_string();
        data.election_subcategory = { if
            data.election_domain == "Noorderzijlvest" ||
            data.election_domain == "Fryslân" ||
            data.election_domain == "Hunze en Aa's" ||
            data.election_domain == "Zuiderzeeland" {
                "AB1".to_string() // Small water board council (less than 19 seats)
            } else {
                "AB2".to_string() // Large water board council
            }
        };
    } else {
        panic!("not yet implemented");
    }
    data.election_identifier_id = {
        let ascii = data.election_domain.replace("â","a");
        let normalized =  re.replace_all(&ascii, "");
        format!("{}{}_{}", &data.election_category, &data.election_date[0..4], &normalized).to_string()
    };

    data.creation_date_time = format!("{}T12:34:56.789", &data.election_date);

    if match iter.peek() { Some(Ok(c)) => c.get(0), _ => None } == Some("") {
        iter.next().unwrap().expect("a CSV row");
    }
    append_row5!(iter, data.naam, "Lijstnummer", "Aanduiding", "Volgnummer", "Naam kandidaat", "Totaal");
    append_row5!(iter, data.gebiednummer, "Gebiednummer", "", "", "", "");
    if match iter.peek() { Some(Ok(c)) => c.get(0), _ => None } == Some("Postcode") {
        append_row5!(iter, data.postcode, "Postcode", "", "", "", "");
    } else {
        for _i in 0..data.gebiednummer.len() {
            data.postcode.push("".to_string());
        }
    }

    append_row4!(iter, data.opgeroepenen, "opgeroepenen");
    append_row4!(iter, data.geldige_stempas, "geldige stempas");
    append_row4!(iter, data.geldig_volmachtbewijs, "geldig volmachtbewijs");
    append_row4!(iter, data.geldige_kiezerspas, "geldige kiezerspas");
    append_row4!(iter, data.toegelaten_kiezers, "toegelaten kiezers");
    append_row4!(iter, data.geldige_stembiljetten, "geldige stembiljetten");
    append_row4!(iter, data.blanco_stembiljetten, "blanco stembiljetten");
    append_row4!(iter, data.ongeldige_stembiljetten, "ongeldige stembiljetten");
    append_row4!(iter, data.aangetroffen_stembiljetten, "aangetroffen stembiljetten");
    append_row4!(iter, data.meer_stembiljetten_dan_toegelaten_kiezers, "meer stembiljetten dan toegelaten kiezers ");
    append_row4!(iter, data.minder_stembiljetten_dan_toegelaten_kiezers, "minder stembiljetten dan toegelaten kiezers ");
    append_row4!(iter, data.kiezers_met_stembiljet_hebben_niet_gestemd, "kiezers met stembiljet hebben niet gestemd");
    append_row4!(iter, data.er_zijn_te_weinig_stembiljetten_uitgereikt, "er zijn te weinig stembiljetten uitgereikt");
    append_row4!(iter, data.er_zijn_te_veel_stembiljetten_uitgereikt, "er zijn te veel stembiljetten uitgereikt");
    append_row4!(iter, data.geen_verklaring, "geen verklaring");
    append_row4!(iter, data.andere_verklaring, "andere verklaring");

    for _i in 0..data.opgeroepenen.len() {
        data.kandidaatstemmen.push(vec![]);
    }
    for result in iter {
        let row = result.expect("a CSV row");
        data.lijstnummer.push(row.get(0).unwrap().to_owned());
        data.partij.push(row.get(1).unwrap().to_owned());
        data.kandidaatnummer.push(row.get(2).unwrap().to_owned());
        data.kandidaatnaam.push(row.get(3).unwrap().to_owned());
        for i in 4..row.len() {
            data.kandidaatstemmen[i-4].push(row.get(i).unwrap().to_owned());
        }
    }
    data
}

fn main() {
    print!("{}", data2eml(csv2data(Box::new(io::stdin()) as Box<Read>)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::fs::File;

    fn test_osv(csv_file: &str, eml_file: &str, correct_ts: &str) {
        let default_ts = "20T12:34:56.789";
        let csv_rdr = Box::new(File::open(csv_file).expect("Could not read csv file")) as Box<Read>;
        assert_eq!(
            data2eml(csv2data(csv_rdr)).replace(default_ts, correct_ts),
            fs::read_to_string(eml_file).expect("Could not read eml file"));
    }

    #[test]
    fn full_integration_ps2019() {
        test_osv(
            "testdata/osv4-3_Telling_PS2019_Fryslan_gemeente_Tytsjerksteradiel.csv",
            "testdata/Telling_PS2019_Fryslan_gemeente_Tytsjerksteradiel.eml.xml",
            "21T06:57:22.834"
        );
    }

    #[test]
    fn full_integration_ab2019() {
        test_osv(
            "testdata/osv4-3_Telling_PS2019_Fryslan_gemeente_Tytsjerksteradiel.csv",
            "testdata/Telling_PS2019_Fryslan_gemeente_Tytsjerksteradiel.eml.xml",
            "21T06:57:22.834"
        );
    }
}
