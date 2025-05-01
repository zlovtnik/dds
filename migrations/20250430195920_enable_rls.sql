-- Enable RLS on all tables
ALTER TABLE public.users ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.jobs ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.tasks ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.pipeline_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.json_data ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.test_table ENABLE ROW LEVEL SECURITY;

-- Create policies for users table
CREATE POLICY "Users can view their own data" ON public.users
    FOR SELECT USING (auth.uid() = id::text);

CREATE POLICY "Users can update their own data" ON public.users
    FOR UPDATE USING (auth.uid() = id::text);

-- Create policies for jobs table
CREATE POLICY "Users can view their own jobs" ON public.jobs
    FOR SELECT USING (auth.uid() = created_by::text);

CREATE POLICY "Users can create their own jobs" ON public.jobs
    FOR INSERT WITH CHECK (auth.uid() = created_by::text);

CREATE POLICY "Users can update their own jobs" ON public.jobs
    FOR UPDATE USING (auth.uid() = created_by::text);

CREATE POLICY "Users can delete their own jobs" ON public.jobs
    FOR DELETE USING (auth.uid() = created_by::text);

-- Create policies for tasks table
CREATE POLICY "Users can view tasks for their jobs" ON public.tasks
    FOR SELECT USING (
        EXISTS (
            SELECT 1 FROM public.jobs
            WHERE jobs.id = tasks.job_id
            AND jobs.created_by::text = auth.uid()
        )
    );

CREATE POLICY "Users can create tasks for their jobs" ON public.tasks
    FOR INSERT WITH CHECK (
        EXISTS (
            SELECT 1 FROM public.jobs
            WHERE jobs.id = tasks.job_id
            AND jobs.created_by::text = auth.uid()
        )
    );

CREATE POLICY "Users can update tasks for their jobs" ON public.tasks
    FOR UPDATE USING (
        EXISTS (
            SELECT 1 FROM public.jobs
            WHERE jobs.id = tasks.job_id
            AND jobs.created_by::text = auth.uid()
        )
    );

CREATE POLICY "Users can delete tasks for their jobs" ON public.tasks
    FOR DELETE USING (
        EXISTS (
            SELECT 1 FROM public.jobs
            WHERE jobs.id = tasks.job_id
            AND jobs.created_by::text = auth.uid()
        )
    );

-- Create policies for pipeline_runs table
CREATE POLICY "Users can view pipeline runs for their jobs" ON public.pipeline_runs
    FOR SELECT USING (
        EXISTS (
            SELECT 1 FROM public.jobs
            WHERE jobs.id = pipeline_runs.job_id
            AND jobs.created_by::text = auth.uid()
        )
    );

CREATE POLICY "Users can create pipeline runs for their jobs" ON public.pipeline_runs
    FOR INSERT WITH CHECK (
        EXISTS (
            SELECT 1 FROM public.jobs
            WHERE jobs.id = pipeline_runs.job_id
            AND jobs.created_by::text = auth.uid()
        )
    );

CREATE POLICY "Users can update pipeline runs for their jobs" ON public.pipeline_runs
    FOR UPDATE USING (
        EXISTS (
            SELECT 1 FROM public.jobs
            WHERE jobs.id = pipeline_runs.job_id
            AND jobs.created_by::text = auth.uid()
        )
    );

CREATE POLICY "Users can delete pipeline runs for their jobs" ON public.pipeline_runs
    FOR DELETE USING (
        EXISTS (
            SELECT 1 FROM public.jobs
            WHERE jobs.id = pipeline_runs.job_id
            AND jobs.created_by::text = auth.uid()
        )
    );

-- Create policies for json_data table
CREATE POLICY "Users can view their own json data" ON public.json_data
    FOR SELECT USING (auth.uid() = user_id::text);

CREATE POLICY "Users can create their own json data" ON public.json_data
    FOR INSERT WITH CHECK (auth.uid() = user_id::text);

CREATE POLICY "Users can update their own json data" ON public.json_data
    FOR UPDATE USING (auth.uid() = user_id::text);

CREATE POLICY "Users can delete their own json data" ON public.json_data
    FOR DELETE USING (auth.uid() = user_id::text);

-- Create policies for test_table
CREATE POLICY "Users can view their own test data" ON public.test_table
    FOR SELECT USING (auth.uid() = user_id::text);

CREATE POLICY "Users can create their own test data" ON public.test_table
    FOR INSERT WITH CHECK (auth.uid() = user_id::text);

CREATE POLICY "Users can update their own test data" ON public.test_table
    FOR UPDATE USING (auth.uid() = user_id::text);

CREATE POLICY "Users can delete their own test data" ON public.test_table
    FOR DELETE USING (auth.uid() = user_id::text);

-- Note: _sqlx_migrations table should remain accessible for migrations
-- No RLS policy needed for _sqlx_migrations as it's managed by SQLx 