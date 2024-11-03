DROP TABLE IF EXISTS f_traces;
CREATE TABLE f_traces
(
    f_id           bigserial PRIMARY KEY,
    f_trace        jsonb                    NOT NULL,
    f_created_at   TIMESTAMP with time zone NOT NULL DEFAULT now()
);
