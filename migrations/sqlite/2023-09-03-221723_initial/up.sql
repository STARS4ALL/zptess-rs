-------------------------------
-- zptess database Data Model
-------------------------------

/* 
 * THIS FILE IS PNLY FOR DOCUMENTATION PURPOSES
 * WE ARE NOT GOING TO RUN IT ANYMORE AS WE HAVE
 * A RUNNING DATABASE
 */


-- This is the database counterpart of a configuration file
-- All configurations are stored here
CREATE TABLE IF NOT EXISTS config_t
(
    section        TEXT NOT NULL,  -- Configuration section
    property       TEXT NOT NULL,  -- Property name
    value          TEXT NOT NULL,  -- Property value

    PRIMARY KEY(section, property)
);

-- batch calibration table
CREATE TABLE IF NOT EXISTS batch_t
(
    begin_tstamp    TIMESTAMP NOT NULL,  -- begin timestamp session
    end_tstamp      TIMESTAMP,  -- end timestamp session
    email_sent      INTEGER,    -- 1=Yes, 0=No, NULL=didn't even try.
    calibrations    INTEGER,    -- number of calibrations performed in this period
    comment         TEXT,       -- optional comment for the opened calibration batch

    PRIMARY KEY(begin_tstamp)
);

-- raw samples table
CREATE TABLE IF NOT EXISTS samples_t
(
    tstamp          TIMESTAMP NOT NULL,  -- sample timestamp
    role            TEXT      NOT NULL,  -- either 'test' or 'ref'
    session         TIMESTAMP,  -- calibration session identifier
    freq            REAL,       -- measured frequency
    seq             INTEGER,    -- sequence number for JSON based raw readings, NULL otherwise
    temp_box        REAL,       -- Box temperature for JSON based raw readings, NULL otherwise
  
    PRIMARY KEY(role, tstamp)
);

-- rounds window table
CREATE TABLE IF NOT EXISTS rounds_t
(
    session         TIMESTAMP NOT NULL,  -- calibration session identifier
    round           INTEGER NOT NULL,    -- to link ref and test windows
    role            TEXT NOT NULL,       -- either 'test' or 'ref'
    begin_tstamp    TIMESTAMP,  -- calibration window start timestamp
    end_tstamp      TIMESTAMP,  -- calibration window end timestamp
    central         TEXT,       -- estimate of central tendency: either 'mean','median' or 'mode'
    freq            REAL,       -- central frequency estimate
    stddev          REAL,       -- Standard deviation for frequency central estimate
    mag             REAL,       -- magnitiude corresponding to central frequency and summing ficticious zero point 
    zp_fict         REAL,       -- Ficticious ZP to estimate instrumental magnitudes (=20.50)
    zero_point      REAL,       -- Estimated Zero Point for this round ('test' photometer round only, else NULL)
    nsamples        INTEGER,    -- Number of samples for this round
    duration        REAL,       -- Approximate duration, in seconds

    PRIMARY KEY(session, role, round)
);

CREATE VIEW IF NOT EXISTS rounds_v 
AS SELECT
    r.session,
    r.round,
    r.role,
    r.begin_tstamp,
    r.end_tstamp,
    r.central,
    r.freq,
    r.stddev,
    r.mag,
    r.zp_fict,
    r.zero_point,
    r.nsamples,
    r.duration,
    s.model,
    s.name,
    s.mac,
    s.nrounds,
    s.upd_flag
FROM rounds_t AS r
JOIN summary_t AS s USING (session, role);

-- Summary calibration table
CREATE TABLE IF NOT EXISTS summary_t
(
    session           TIMESTAMP NOT NULL,  -- calibration session identifier
    role              TEXT NOT NULL,       -- either 'test' or 'ref'
    calibration       TEXT,       -- either 'MANUAL' or 'AUTO'
    calversion        TEXT,       -- calibration software version
    model             TEXT,  -- TESS model
    name              TEXT,  -- TESS name
    mac               TEXT,  -- TESS MAC address
    firmware          TEXT,  -- firmware revision
    sensor            TEXT,  -- Sensor model (TSL237, S9705-01DT)
    prev_zp           REAL,  -- previous ZP before calibration
    author            TEXT,  -- who run the calibration
    nrounds           INTEGER, -- Number of rounds passed
    offset            REAL,  -- Additional offset that was summed to the computed zero_point
    upd_flag          INTEGER, -- 1 => TESS-W ZP was updated, 0 => TESS-W ZP was not updated,
    zero_point        REAL,  -- calibrated zero point
    zero_point_method TEXT,  -- either the 'mode' or 'median' of the different rounds
    freq              REAL,  -- final chosen frequency
    freq_method       TEXT,  -- either the 'mode' or 'median' of the different rounds
    mag               REAL,  -- final chosen magnitude uzing ficticious ZP
    filter            TEXT,  -- Filter type (i.e. UV-IR/740)
    plug              TEXT,  -- Plug type (i.e. USB-A)
    box               TEXT,  -- Box model (i.e. FSH714)
    collector         TEXT,  -- Collector model
    comment           TEXT,  -- Additional comment for the callibration process
    PRIMARY KEY(session, role)
);


CREATE VIEW IF NOT EXISTS summary_v 
AS SELECT
    test_t.session,
    test_t.role,
    test_t.calibration,
    test_t.calversion,
    test_t.model,
    test_t.name,
    test_t.mac,
    test_t.firmware,
    test_t.sensor,
    test_t.prev_zp,
    test_t.author,
    test_t.nrounds,
    test_t.offset,
    test_t.upd_flag,
    ROUND(test_t.zero_point, 2) AS zero_point,
    test_t.zero_point_method,
    ROUND(test_t.freq,3)        AS test_freq,
    test_t.freq_method          AS test_freq_method,
    ROUND(test_t.mag, 2)        AS test_mag,
    ROUND(ref_t.freq, 3)        AS ref_freq,
    ref_t.freq_method           AS ref_freq_method,
    ROUND(ref_t.mag, 2)         AS ref_mag,
    ROUND(ref_t.mag - test_t.mag, 2) AS mag_diff,
    ROUND(test_t.zero_point, 2) - test_t.offset as raw_zero_point,
    test_t.filter,
    test_t.plug,
    test_t.box,
    test_t.collector,
    test_t.comment

FROM summary_t AS ref_t
JOIN summary_t AS test_t USING (session)
WHERE test_t.role = 'test' AND ref_t.role = 'ref';


--------------------------------------------------------
-- Miscelaneous data to be inserted at database creation
--------------------------------------------------------

-- Global section

INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('database', 'version', '07');

------------------------------
-- Calibration process section
------------------------------

INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('calibration', 'rounds', '5');
INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('calibration', 'offset', '0.0');
INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('calibration', 'author', '');
INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('calibration', 'zp_fict', '20.50');

-------------------------------
-- Reference photometer section
-------------------------------

-- Default device identification values when using serial line

INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('ref-device', 'model', 'TESS-W');
INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('ref-device', 'name', 'stars3');
INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('ref-device', 'mac', '18:FE:34:CF:E9:A3');
INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('ref-device', 'firmware', '');
INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('ref-device', 'sensor', 'TSL237');
INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('ref-device', 'zp', '20.44');
INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('ref-device', 'freq_offset', '0.0');

-- Default device protocol and comm method

INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('ref-device', 'endpoint', 'serial:/dev/ttyUSB0:9600');
INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('ref-device', 'old_proto', '1');

-- Default statistics to compute

INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('ref-stats', 'samples', '125');
INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('ref-stats', 'period', '5');
INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('ref-stats', 'central', 'median');

-------------------------------
--  Test photometer section
-------------------------------

-- Default device identification

INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('test-device', 'model', 'TESS-W');
INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('test-device', 'sensor', 'TSL237');
-- Default device protocol and comm method

INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('test-device', 'endpoint', 'udp:192.168.4.1:2255');
INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('test-device', 'old_proto', '0');

-- Default statistics to compute

INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('test-stats', 'samples', '125');
INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('test-stats', 'period', '5');
INSERT OR REPLACE INTO config_t(section, property, value) 
VALUES ('test-stats', 'central', 'median');

--------------------------------------------------------
-- Miscelaneous data to be inserted at database creation
--------------------------------------------------------

-- Batches manually compiled froM zptess.scv

INSERT OR REPLACE INTO batch_t(begin_tstamp, end_tstamp, calibrations) 
VALUES ('2019-06-10T12:42:31', '2019-06-18T16:54:10', 43);

INSERT OR REPLACE INTO batch_t(begin_tstamp, end_tstamp, calibrations) 
VALUES ('2019-09-11T10:34:43', '2019-09-11T10:36:51', 2);

INSERT OR REPLACE INTO batch_t(begin_tstamp, end_tstamp, calibrations) 
VALUES ('2019-09-26T17:21:22', '2019-10-02T09:51:09', 122);

INSERT OR REPLACE INTO batch_t(begin_tstamp, end_tstamp, calibrations) 
VALUES ('2019-10-28T09:37:00', '2019-10-29T15:16:18', 48);

INSERT OR REPLACE INTO batch_t(begin_tstamp, end_tstamp, calibrations) 
VALUES ('2019-11-11T09:19:08', '2019-11-11T11:04:51', 42);

INSERT OR REPLACE INTO batch_t(begin_tstamp, end_tstamp, calibrations) 
VALUES ('2019-12-02T09:31:30', '2019-12-03T09:54:43', 3);

INSERT OR REPLACE INTO batch_t(begin_tstamp, end_tstamp, calibrations) 
VALUES ('2020-01-21T11:04:20', '2020-01-22T09:29:12', 77);

INSERT OR REPLACE INTO batch_t(begin_tstamp, end_tstamp, calibrations) 
VALUES ('2020-02-25T09:29:33', '2020-02-27T11:17:43', 8);

INSERT OR REPLACE INTO batch_t(begin_tstamp, end_tstamp, calibrations) 
VALUES ('2020-03-10T14:40:04', '2020-03-10T14:44:57', 3);

INSERT OR REPLACE INTO batch_t(begin_tstamp, end_tstamp, calibrations) 
VALUES ('2020-07-02T08:14:44', '2020-07-02T10:51:25', 31);

INSERT OR REPLACE INTO batch_t(begin_tstamp, end_tstamp, calibrations) 
VALUES ('2020-07-09T08:04:43', '2020-07-09T09:50:32', 23);

INSERT OR REPLACE INTO batch_t(begin_tstamp, end_tstamp, calibrations) 
VALUES ('2020-10-06T08:55:48', '2020-10-07T08:56:23', 21);

INSERT OR REPLACE INTO batch_t(begin_tstamp, end_tstamp, calibrations) 
VALUES ('2020-12-22T09:06:03', '2020-12-22T10:08:51', 18);

INSERT OR REPLACE INTO batch_t(begin_tstamp, end_tstamp, calibrations) 
VALUES ('2021-02-05T10:12:33', '2021-02-05T10:26:48', 5);

INSERT OR REPLACE INTO batch_t(begin_tstamp, end_tstamp, calibrations) 
VALUES ('2021-02-23T09:18:21', '2021-02-23T11:18:00', 15);

INSERT OR REPLACE INTO batch_t(begin_tstamp, end_tstamp, calibrations) 
VALUES ('2021-05-26T07:48:43', '2021-06-02T08:14:29', 61);

INSERT OR REPLACE INTO batch_t(begin_tstamp, end_tstamp, calibrations) 
VALUES ('2021-08-31T09:22:17', '2021-08-31T11:50:58', 36);

INSERT OR REPLACE INTO batch_t(begin_tstamp, end_tstamp, calibrations) 
VALUES ('2021-09-15T08:09:34', '2021-09-15T09:12:18', 4);

INSERT OR REPLACE INTO batch_t(begin_tstamp, end_tstamp, calibrations) 
VALUES ('2021-09-28T09:19:49', '2021-09-28T09:24:56', 2);

INSERT OR REPLACE INTO batch_t(begin_tstamp, end_tstamp, calibrations) 
VALUES ('2021-10-13T09:06:10', '2021-10-13T10:07:11', 6);

