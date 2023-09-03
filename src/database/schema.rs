// @generated automatically by Diesel CLI.

diesel::table! {
    batch_t (begin_tstamp) {
        begin_tstamp -> Timestamp,
        end_tstamp -> Nullable<Timestamp>,
        email_sent -> Nullable<Integer>,
        calibrations -> Nullable<Integer>,
        comment -> Nullable<Text>,
    }
}

diesel::table! {
    config_t (section, property) {
        section -> Text,
        property -> Text,
        value -> Text,
    }
}

diesel::table! {
    rounds_t (session, round, role) {
        session -> Nullable<Timestamp>,
        round -> Nullable<Integer>,
        role -> Nullable<Text>,
        begin_tstamp -> Nullable<Timestamp>,
        end_tstamp -> Nullable<Timestamp>,
        central -> Nullable<Text>,
        freq -> Nullable<Float>,
        stddev -> Nullable<Float>,
        mag -> Nullable<Float>,
        zp_fict -> Nullable<Float>,
        zero_point -> Nullable<Float>,
        nsamples -> Nullable<Integer>,
        duration -> Nullable<Float>,
    }
}

diesel::table! {
    samples_t (tstamp, role) {
        tstamp -> Nullable<Timestamp>,
        role -> Nullable<Text>,
        session -> Nullable<Timestamp>,
        freq -> Nullable<Float>,
        seq -> Nullable<Integer>,
        temp_box -> Nullable<Float>,
    }
}

diesel::table! {
    summary_t (session, role) {
        session -> Nullable<Timestamp>,
        role -> Nullable<Text>,
        model -> Nullable<Text>,
        name -> Nullable<Text>,
        mac -> Nullable<Text>,
        firmware -> Nullable<Text>,
        prev_zp -> Nullable<Float>,
        author -> Nullable<Text>,
        nrounds -> Nullable<Integer>,
        offset -> Nullable<Float>,
        upd_flag -> Nullable<Integer>,
        zero_point -> Nullable<Float>,
        zero_point_method -> Nullable<Text>,
        freq -> Nullable<Float>,
        freq_method -> Nullable<Text>,
        mag -> Nullable<Float>,
        calibration -> Nullable<Text>,
        filter -> Nullable<Text>,
        plug -> Nullable<Text>,
        #[sql_name = "box"]
        box_ -> Nullable<Text>,
        collector -> Nullable<Text>,
        comment -> Nullable<Text>,
        sensor -> Nullable<Text>,
        calversion -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    batch_t,
    config_t,
    rounds_t,
    samples_t,
    summary_t,
);
