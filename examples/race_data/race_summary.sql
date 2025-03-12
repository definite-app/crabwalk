-- Race Summary Table
-- This query creates a summary of race performance metrics by driver

WITH 
-- Convert lap times from string format to seconds for calculations
lap_times_in_seconds AS (
    SELECT 
        DRIVER_NAME,
        TEAM,
        MANUFACTURER,
        "CLASS",
        LAP_NUMBER,
        -- Convert lap time from MM:SS.sss format to seconds using DuckDB string functions
        CASE 
            WHEN LAP_TIME LIKE '%:%' THEN 
                -- Extract minutes (before colon) and convert to seconds
                (TRY_CAST(SPLIT_PART(LAP_TIME, ':', 1) AS DOUBLE) * 60) + 
                -- Extract seconds part (after colon)
                TRY_CAST(SPLIT_PART(LAP_TIME, ':', 2) AS DOUBLE)
            ELSE TRY_CAST(LAP_TIME AS DOUBLE)
        END AS lap_time_seconds,
        KPH,
        TOP_SPEED,
        PIT_TIME
    FROM transform.races
    WHERE LAP_TIME IS NOT NULL AND LAP_TIME != ''
),

-- Calculate best lap times and averages
driver_stats AS (
    SELECT
        DRIVER_NAME,
        TEAM,
        MANUFACTURER,
        "CLASS",
        COUNT(DISTINCT LAP_NUMBER) AS total_laps,
        MIN(lap_time_seconds) AS best_lap_time_seconds,
        AVG(lap_time_seconds) AS avg_lap_time_seconds,
        MAX(KPH) AS max_speed_kph,
        AVG(KPH) AS avg_speed_kph,
        COUNT(PIT_TIME) AS pit_stops
    FROM lap_times_in_seconds
    GROUP BY DRIVER_NAME, TEAM, MANUFACTURER, "CLASS"
)

-- Final summary table
SELECT
    DRIVER_NAME,
    TEAM,
    MANUFACTURER,
    "CLASS",
    total_laps,
    -- Format best lap time back to MM:SS.sss using DuckDB's formatting
    CONCAT(
        CAST(FLOOR(best_lap_time_seconds / 60) AS INTEGER),
        ':',
        LPAD(ROUND(CAST(best_lap_time_seconds % 60 AS DECIMAL(10,3)), 3)::VARCHAR, 6, '0')
    ) AS best_lap_time,
    -- Format average lap time back to MM:SS.sss
    CONCAT(
        CAST(FLOOR(avg_lap_time_seconds / 60) AS INTEGER),
        ':',
        LPAD(ROUND(CAST(avg_lap_time_seconds % 60 AS DECIMAL(10,3)), 3)::VARCHAR, 6, '0')
    ) AS avg_lap_time,
    ROUND(max_speed_kph, 1) AS max_speed_kph,
    ROUND(avg_speed_kph, 1) AS avg_speed_kph,
    pit_stops,
    -- Calculate position within class based on best lap time
    ROW_NUMBER() OVER (PARTITION BY "CLASS" ORDER BY best_lap_time_seconds) AS position_in_class,
    -- Calculate overall position based on best lap time
    ROW_NUMBER() OVER (ORDER BY best_lap_time_seconds) AS overall_position
FROM driver_stats
ORDER BY best_lap_time_seconds;
