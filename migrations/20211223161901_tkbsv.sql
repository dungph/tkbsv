CREATE TABLE public.student_schedule (
    student_code text PRIMARY KEY,
    schedule_data jsonb NOT NULL DEFAULT '{}'::jsonb,
    CONSTRAINT check_data_is_object CHECK (jsonb_typeof(schedule_data) = 'object')
);
