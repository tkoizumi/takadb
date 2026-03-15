use serde::Deserialize;
#[derive(Deserialize, Debug)]
pub struct RawRecord {
    #[serde(alias = "Unique Key")]
    pub unique_key: String,
    #[serde(alias = "Created Date")]
    pub created_date: String,
    #[serde(alias = "Closed Date")]
    pub closed_date: String,
    #[serde(alias = "Agency")]
    pub agency: String,
    #[serde(alias = "Agency Name")]
    pub agency_name: String,
    #[serde(alias = "Problem (formerly Complaint Type)")]
    pub complaint_type: String,
    #[serde(alias = "Problem Detail (formerly Descriptor)")]
    pub descriptor: String,
    #[serde(alias = "Location Type")]
    pub location_type: String,
    #[serde(alias = "Incident Zip")]
    pub incident_zip: String,
    #[serde(alias = "Incident Address")]
    pub incident_address: String,
    #[serde(alias = "Street Name")]
    pub street_name: String,
    #[serde(alias = "Cross Street 1")]
    pub cross_street_1: String,
    #[serde(alias = "Cross Street 2")]
    pub cross_street_2: String,
    #[serde(alias = "Intersection Street 1")]
    pub intersection_street_1: String,
    #[serde(alias = "Intersection Street 2")]
    pub intersection_street_2: String,
    #[serde(alias = "Address Type")]
    pub address_type: String,
    #[serde(alias = "City")]
    pub city: String,
    #[serde(alias = "Landmark")]
    pub landmark: String,
    #[serde(alias = "Facility Type")]
    pub facility_type: String,
    #[serde(alias = "Status")]
    pub status: String,
    #[serde(alias = "Due Date")]
    pub due_date: String,
    #[serde(alias = "Resolution Action Updated Date")]
    pub resolution_action_updated_date: String,
    #[serde(alias = "Community Board")]
    pub community_board: String,
    #[serde(alias = "Borough")]
    pub borough: String,
    #[serde(alias = "X Coordinate (State Plane)")]
    pub x_coordinate_state_plane: String,
    #[serde(alias = "Y Coordinate (State Plane)")]
    pub y_coordinate_state_plane: String,
    #[serde(alias = "Park Facility Name")]
    pub park_facility_name: String,
    #[serde(alias = "Park Borough")]
    pub park_borough: String,
    #[serde(alias = "Vehicle Type")]
    pub vehicle_type: String,
    #[serde(alias = "Taxi Company Borough")]
    pub taxi_company_borough: String,
    #[serde(alias = "Taxi Pick Up Location")]
    pub taxi_pick_up_location: String,
    #[serde(alias = "Bridge Highway Name")]
    pub bridge_highway_name: String,
    #[serde(alias = "Bridge Highway Direction")]
    pub bridge_highway_direction: String,
    #[serde(alias = "Road Ramp")]
    pub road_ramp: String,
    #[serde(alias = "Bridge Highway Segment")]
    pub bridge_highway_segment: String,
    #[serde(alias = "Latitude")]
    pub latitude: String,
    #[serde(alias = "Longitude")]
    pub longitude: String,
    #[serde(alias = "Location")]
    pub location: String,
}
