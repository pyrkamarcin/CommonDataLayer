namespace CDL.Tests.ServiceObjects.SchemaService
{
    public class ComputedFiled
    {        
        public Computed computed { get; set; }
    }

    public class Computed
    {        
        public Computation computation { get; set; }
        public string field_type { get; set; }
    }

    public class Computation
    {
        public FieldValueComputation field_value { get; set; }
    }

    public class FieldValueComputation
    {
        public uint schema_id { get; set; }
        public string  field_path { get; set; }
    }

    public enum FieldType {
        String,
        Numeric,
        Json
    }
}