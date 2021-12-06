use oxigraph::model::NamedNodeRef;

pub const HOST_PREFIX: &str = "https://data.iotics.com/diddy/hosts/";

pub const HOST: NamedNodeRef<'_> =
    NamedNodeRef::new_unchecked("https://data.iotics.com/diddy/Host");
pub const HOST_ADDRESS: NamedNodeRef<'_> =
    NamedNodeRef::new_unchecked("https://data.iotics.com/diddy/address");
pub const DID_KEY_NAME: NamedNodeRef<'_> =
    NamedNodeRef::new_unchecked("https://data.iotics.com/diddy/keyName");
pub const TWIN: NamedNodeRef<'_> =
    NamedNodeRef::new_unchecked("https://data.iotics.com/diddy/Twin");
pub const DID_UPDATE_TIME: NamedNodeRef<'_> =
    NamedNodeRef::new_unchecked("https://data.iotics.com/diddy/updateTime");
pub const TWIN_LIVES_IN: NamedNodeRef<'_> =
    NamedNodeRef::new_unchecked("https://data.iotics.com/diddy/livesIn");
pub const TWIN_CONTROLLED_BY: NamedNodeRef<'_> =
    NamedNodeRef::new_unchecked("https://data.iotics.com/diddy/controlledBy");
pub const AGENT: NamedNodeRef<'_> =
    NamedNodeRef::new_unchecked("https://data.iotics.com/diddy/Agent");

pub const ONTOLOGY: &str = r#"
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix diddy: <https://data.iotics.com/diddy/> .

diddy:Host rdf:type rdf:Class .

diddy:address rdf:type rdf:Property ;
        rdfs:label "Host Address"@en ;
        rdfs:domain diddy:Host ;
        rdfs:range xsd:string .

diddy:DID rdf:type rdf:Class .

diddy:keyName rdf:type rdf:Property ;
        rdfs:label "Key Name"@en ;
        rdfs:domain diddy:DID ;
        rdfs:range xsd:string .

diddy:updateTime rdf:type rdf:Property ;
        rdfs:label "Update Time"@en ;
        rdfs:domain diddy:DID ;
        rdfs:range xsd:dateTime .

diddy:Twin rdf:type rdf:Class ;
        rdfs:subClassOf diddy:DID .

diddy:livesIn rdf:type rdf:Property ;
        rdfs:label "Lives In"@en ;
        rdfs:domain diddy:Twin ;
        rdfs:range diddy:Host .

diddy:controlledBy rdf:type rdf:Property ;
        rdfs:label "Controlled By"@en ;
        rdfs:domain diddy:Twin ;
        rdfs:range diddy:Agent .

diddy:Agent rdf:type rdf:Class ;
        rdfs:subClassOf diddy:DID .
"#;
