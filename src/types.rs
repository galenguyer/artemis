#![allow(dead_code, non_snake_case)]

use crate::fcc_date;
use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Amateur<'a> {
    pub RecordType: &'a str,
    pub UniqueSystemIdentifier: u32,
    pub UlsFileNumber: &'a str,
    pub EBFNumber: &'a str,
    pub CallSign: &'a str,
    pub OperatorClass: &'a str,
    pub GroupCode: &'a str,
    pub RegionCode: &'a str,
    pub TrusteeCallSign: &'a str,
    pub TrusteeIndicator: &'a str,
    pub PhysicianCertification: &'a str,
    pub VESignature: &'a str,
    pub SystematicCallSignChange: &'a str,
    pub VanityCallSignChange: &'a str,
    pub VanityRelationship: &'a str,
    pub PreviousCallSign: &'a str,
    pub PreviousOperatorClass: &'a str,
    pub TrusteeName: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct Comment<'a> {
    pub RecordType: &'a str,
    pub UniqueSystemIdentifier: &'a str,
    pub UlsFileNumber: &'a str,
    pub CallSign: &'a str,
    #[serde(with = "fcc_date")]
    pub CommentDate: Option<NaiveDate>,
    pub Description: &'a str,
    pub StatusCode: &'a str,
    #[serde(with = "fcc_date")]
    pub StatusDate: Option<NaiveDate>,
}

#[derive(Deserialize, Debug)]
pub struct Entity<'a> {
    pub RecordType: &'a str,
    pub UniqueSystemIdentifier: u32,
    pub UlsFileNumber: &'a str,
    pub EBFNumber: &'a str,
    pub CallSign: &'a str,
    pub EntityType: &'a str,
    pub LicenseeId: &'a str,
    pub EntityName: &'a str,
    pub FirstName: &'a str,
    pub MiddleInitial: &'a str,
    pub LastName: &'a str,
    pub Suffix: &'a str,
    pub Phone: &'a str,
    pub Fax: &'a str,
    pub Email: &'a str,
    pub StreetAddress: &'a str,
    pub City: &'a str,
    pub State: &'a str,
    pub ZipCode: &'a str,
    pub POBox: &'a str,
    pub AttentionLine: &'a str,
    pub SGIN: &'a str,
    pub FRN: &'a str,
    pub ApplicantTypeCode: &'a str,
    pub ApplicantTypeCodeOther: &'a str,
    pub StatusCode: &'a str,
    #[serde(with = "fcc_date")]
    pub StatusDate: Option<NaiveDate>,
    pub ThreePointSevenGhzLicenseType: &'a str,
    pub LinkedUniqueSystemIdentifier: &'a str,
    pub LinkedCallsign: &'a str,
}

#[derive(Deserialize, Debug)]
pub struct Header<'a> {
    pub RecordType: &'a str,
    pub UniqueSystemIdentifier: u32,
    pub UlsFileNumber: &'a str,
    pub EBFNumber: &'a str,
    pub CallSign: &'a str,
    pub LicenseStatus: &'a str,
    pub RadioServiceCode: &'a str,
    #[serde(with = "fcc_date")]
    pub GrantDate: Option<NaiveDate>,
    #[serde(with = "fcc_date")]
    pub ExpiredDate: Option<NaiveDate>,
    #[serde(with = "fcc_date")]
    pub CancellationDate: Option<NaiveDate>,
    pub EligibilityRuleNumber: &'a str,
    pub Reserved: &'a str,
    pub Alien: &'a str,
    pub AlienGovernment: &'a str,
    pub AlienCorporation: &'a str,
    pub AlienOfficers: &'a str,
    pub AlienControl: &'a str,
    pub Revoked: &'a str,
    pub Convicted: &'a str,
    pub Adjudged: &'a str,
    pub Reserved2: &'a str,
    pub CommonCarrier: &'a str,
    pub NonCommonCarrier: &'a str,
    pub PrivateComm: &'a str,
    pub Fixed: &'a str,
    pub Mobile: &'a str,
    pub Radiolocation: &'a str,
    pub Sattelite: &'a str,
    pub DevelopmentalOrSta: &'a str,
    pub InterconnectedService: &'a str,
    pub CertifierFirstName: &'a str,
    pub CertifierMiddleInitial: &'a str,
    pub CertifierLastName: &'a str,
    pub CertifierSuffix: &'a str,
    pub CertifierTitle: &'a str,
    pub Female: &'a str,
    pub BlackOrAfricanAmerican: &'a str,
    pub NativeAmerican: &'a str,
    pub Hawaiian: &'a str,
    pub Asian: &'a str,
    pub White: &'a str,
    pub Hispanic: &'a str,
    #[serde(with = "fcc_date")]
    pub EffectiveDate: Option<NaiveDate>,
    #[serde(with = "fcc_date")]
    pub LastActionDate: Option<NaiveDate>,
    pub AuctionId: Option<i32>,
    pub BroadcastServicesRegulatoryStatus: &'a str,
    pub BandManagerRegulatoryStatus: &'a str,
    pub BroadcastServicesTypeOfRadioService: &'a str,
    pub AlienRuling: &'a str,
    pub LicenseeNameChange: &'a str,
    pub WhitespaceIndicator: &'a str,
    pub OperationRequirementChoice: &'a str,
    pub OperationRequirementAnswer: &'a str,
    pub DiscontinuationOfService: &'a str,
    pub RegulatoryCompliance: &'a str,
    pub EligibilityCertification900Mhz: &'a str,
    pub TransitionPlanCertification900Mhz: &'a str,
    pub ReturnSpectrumCertification900Mhz: &'a str,
    pub PaymentCertification900Mhz: &'a str,
}

#[derive(Deserialize, Debug)]
pub struct History<'a> {
    pub RecordType: &'a str,
    pub UniqueSystemIdentifier: &'a str,
    pub UlsFileNumber: &'a str,
    pub CallSign: &'a str,
    #[serde(with = "fcc_date")]
    pub LogDate: Option<NaiveDate>,
    pub Code: &'a str,
}

#[derive(Deserialize, Debug)]
pub struct LicenseAttachment<'a> {
    pub RecordType: &'a str,
    pub UniqueSystemIdentifier: u32,
    pub CallSign: &'a str,
    pub AttachmentCode: &'a str,
    pub AttachmentDescription: &'a str,
    #[serde(with = "fcc_date")]
    pub AttachmentDate: Option<NaiveDate>,
    pub AttachmentFileName: &'a str,
    pub ActionPerformed: &'a str,
}
#[allow(dead_code, non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct SpecialCondition<'a> {
    pub RecordType: &'a str,
    pub UniqueSystemIdentifier: u32,
    pub UlsFileNumber: &'a str,
    pub EBFNumber: &'a str,
    pub CallSign: &'a str,
    pub SpecialConditionType: &'a str,
    pub SpecialConditionCode: Option<i32>,
    pub StatusCode: &'a str,
    #[serde(with = "fcc_date")]
    pub StatusDate: Option<NaiveDate>,
}

#[derive(Deserialize, Debug)]
pub struct SpecialConditionFreeForm<'a> {
    pub RecordType: &'a str,
    pub UniqueSystemIdentifier: u32,
    pub UlsFileNumber: &'a str,
    pub EBFNumber: &'a str,
    pub CallSign: &'a str,
    pub LicenseFreeFormType: &'a str,
    pub UniqueLicenseFreeFormIdentifier: &'a str,
    pub SequenceNumber: Option<i32>,
    pub LicenseFreeFormCondition: &'a str,
    pub StatusCode: &'a str,
    #[serde(with = "fcc_date")]
    pub StatusDate: Option<NaiveDate>,
}
