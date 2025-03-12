-- Driver Fact Table
-- Comprehensive statistics for each driver across all races

WITH 
-- Get driver lap data with converted lap times
driver_lap_data AS (
    SELECT 
        DRIVER_NAME,
        TEAM,
        MANUFACTURER,
        "CLASS",
        LAP_NUMBER,
        -- Convert lap time from MM:SS.sss format to seconds
        CASE 
            WHEN LAP_TIME LIKE '%:%' THEN 
                (TRY_CAST(SPLIT_PART(LAP_TIME, ':', 1) AS DOUBLE) * 60) + 
                TRY_CAST(SPLIT_PART(LAP_TIME, ':', 2) AS DOUBLE)
            ELSE TRY_CAST(LAP_TIME AS DOUBLE)
        END AS lap_time_seconds,
        KPH,
        TOP_SPEED,
        PIT_TIME,
        FLAG_AT_FL
    FROM transform.races
    WHERE LAP_TIME IS NOT NULL AND LAP_TIME != ''
),

-- Get max lap number for each driver (to identify last lap)
driver_max_laps AS (
    SELECT
        DRIVER_NAME,
        MAX(LAP_NUMBER) AS max_lap_number
    FROM driver_lap_data
    GROUP BY DRIVER_NAME
),

-- Get first and last lap times
driver_first_last_laps AS (
    SELECT
        d.DRIVER_NAME,
        -- First lap time
        MIN(CASE WHEN d.LAP_NUMBER = 1 THEN d.lap_time_seconds END) AS first_lap_time,
        -- Last lap time (using the max lap number we calculated)
        MIN(CASE WHEN d.LAP_NUMBER = m.max_lap_number THEN d.lap_time_seconds END) AS last_lap_time
    FROM driver_lap_data d
    JOIN driver_max_laps m ON d.DRIVER_NAME = m.DRIVER_NAME
    GROUP BY d.DRIVER_NAME
),

-- Calculate driver-specific metrics
driver_metrics AS (
    SELECT
        d.DRIVER_NAME,
        d.TEAM,
        d.MANUFACTURER,
        d."CLASS",
        COUNT(DISTINCT d.LAP_NUMBER) AS total_laps,
        MIN(d.lap_time_seconds) AS best_lap_time_seconds,
        AVG(d.lap_time_seconds) AS avg_lap_time_seconds,
        STDDEV(d.lap_time_seconds) AS lap_time_stddev,
        MAX(d.KPH) AS max_speed_kph,
        AVG(d.KPH) AS avg_speed_kph,
        COUNT(d.PIT_TIME) AS pit_stops,
        -- Count laps under different flag conditions
        COUNT(CASE WHEN d.FLAG_AT_FL = 'GF' THEN 1 END) AS green_flag_laps,
        COUNT(CASE WHEN d.FLAG_AT_FL = 'YF' THEN 1 END) AS yellow_flag_laps,
        -- Calculate consistency metrics
        PERCENTILE_CONT(0.25) WITHIN GROUP (ORDER BY d.lap_time_seconds) AS lap_time_p25,
        PERCENTILE_CONT(0.75) WITHIN GROUP (ORDER BY d.lap_time_seconds) AS lap_time_p75,
        -- Add first and last lap times
        fl.first_lap_time,
        fl.last_lap_time
    FROM driver_lap_data d
    LEFT JOIN driver_first_last_laps fl ON d.DRIVER_NAME = fl.DRIVER_NAME
    GROUP BY d.DRIVER_NAME, d.TEAM, d.MANUFACTURER, d."CLASS", fl.first_lap_time, fl.last_lap_time
),

-- Calculate driver rankings
driver_rankings AS (
    SELECT
        DRIVER_NAME,
        "CLASS",
        -- Rank by best lap time within class
        ROW_NUMBER() OVER (PARTITION BY "CLASS" ORDER BY best_lap_time_seconds) AS position_in_class,
        -- Rank by best lap time overall
        ROW_NUMBER() OVER (ORDER BY best_lap_time_seconds) AS overall_position,
        -- Rank by consistency (lower stddev is better)
        ROW_NUMBER() OVER (PARTITION BY "CLASS" ORDER BY lap_time_stddev) AS consistency_rank_in_class,
        -- Rank by average speed
        ROW_NUMBER() OVER (PARTITION BY "CLASS" ORDER BY avg_speed_kph DESC) AS speed_rank_in_class
    FROM driver_metrics
)

-- Final driver fact table
SELECT
    d.DRIVER_NAME,
    d.TEAM,
    d.MANUFACTURER,
    d."CLASS",
    d.total_laps,
    -- Format best lap time as MM:SS.sss
    CONCAT(
        CAST(FLOOR(d.best_lap_time_seconds / 60) AS INTEGER),
        ':',
        LPAD(ROUND(CAST(d.best_lap_time_seconds % 60 AS DECIMAL(10,3)), 3)::VARCHAR, 6, '0')
    ) AS best_lap_time,
    -- Format average lap time as MM:SS.sss
    CONCAT(
        CAST(FLOOR(d.avg_lap_time_seconds / 60) AS INTEGER),
        ':',
        LPAD(ROUND(CAST(d.avg_lap_time_seconds % 60 AS DECIMAL(10,3)), 3)::VARCHAR, 6, '0')
    ) AS avg_lap_time,
    ROUND(d.lap_time_stddev, 3) AS lap_time_stddev,
    -- Calculate interquartile range for consistency
    ROUND(d.lap_time_p75 - d.lap_time_p25, 3) AS lap_time_iqr,
    -- Calculate improvement percentage
    CASE 
        WHEN d.first_lap_time IS NOT NULL AND d.last_lap_time IS NOT NULL AND d.first_lap_time > 0 
        THEN ROUND(((d.first_lap_time - d.last_lap_time) / d.first_lap_time) * 100, 2)
        ELSE NULL
    END AS improvement_percentage,
    ROUND(d.max_speed_kph, 1) AS max_speed_kph,
    ROUND(d.avg_speed_kph, 1) AS avg_speed_kph,
    d.pit_stops,
    d.green_flag_laps,
    d.yellow_flag_laps,
    -- Calculate green flag percentage
    ROUND((d.green_flag_laps::FLOAT / NULLIF(d.total_laps, 0)) * 100, 1) AS green_flag_percentage,
    -- Add rankings
    r.position_in_class,
    r.overall_position,
    r.consistency_rank_in_class,
    r.speed_rank_in_class
FROM driver_metrics d
JOIN driver_rankings r ON d.DRIVER_NAME = r.DRIVER_NAME AND d."CLASS" = r."CLASS"
ORDER BY r.overall_position; 