use oxigraph::model::NamedNodeRef;

pub const PREFIX: &str = "https://data.iotics.com/diddy/";

pub const HOST: NamedNodeRef<'_> =
    NamedNodeRef::new_unchecked("https://data.iotics.com/diddy/Host");
pub const HOST_ADDRESS: NamedNodeRef<'_> =
    NamedNodeRef::new_unchecked("https://data.iotics.com/diddy/Address");

pub const ONTOLOGY: &str = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix diddy: <https://data.iotics.com/diddy/> .

diddy:Host rdf:type rdf:Class .

diddy:Address rdf:type rdf:Property ;
        rdfs:label "Host Address"@en ;
        rdfs:domain diddy:Host ;
        rdfs:range xsd:string .
"#;
