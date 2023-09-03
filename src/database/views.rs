diesel::table! {
    rounds_v (session, round, role) {
        session -> Timestamp,
        round -> Integer,
        role -> Text,
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
        model -> Nullable<Text>,
        name -> Nullable<Text>,
        mac -> Nullable<Text>,
        nrounds -> Nullable<Integer>,
        upd_flag -> Nullable<Integer>,
    }
}

diesel::table! {
    summary_v (session, role) {
        session -> Timestamp,
        role -> Text,
        calibration -> Nullable<Text>,
        calversion -> Nullable<Text>,
        model -> Nullable<Text>,
        name -> Nullable<Text>,
        mac -> Nullable<Text>,
        firmware -> Nullable<Text>,
        sensor -> Nullable<Text>,
        prev_zp -> Nullable<Float>,
        author -> Nullable<Text>,
        nrounds -> Nullable<Integer>,
        offset -> Nullable<Float>,
        upd_flag -> Nullable<Integer>,
        zero_point -> Nullable<Float>,
        zero_point_method -> Nullable<Text>,

        test_freq -> Nullable<Float>,
        test_freq_method -> Nullable<Text>,
        test_mag -> Nullable<Float>,

        ref_freq -> Nullable<Float>,
        ref_freq_method -> Nullable<Text>,
        ref_mag -> Nullable<Float>,

        mag_diff -> Nullable<Float>,
        raw_zero_point -> Nullable<Float>,

        filter -> Nullable<Text>,
        plug -> Nullable<Text>,
        #[sql_name = "box"]
        box_ -> Nullable<Text>,
        collector -> Nullable<Text>,
        comment -> Nullable<Text>,
    }
}
