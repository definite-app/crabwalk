-- @config: {output: {type: "parquet", location: "./output/sample.parquet"}}

select *
from races
limit 20;