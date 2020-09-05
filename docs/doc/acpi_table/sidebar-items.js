initSidebarItems({"struct":[["AcpiTables","The struct holding all ACPI tables and records of where they exist in memory. All ACPI tables are covered by a single large MappedPages object,  which is necessary because they may span multiple pages/frames, and generally should not be multiply aliased/accessed due to potential race conditions. As more ACPI tables are discovered, the single MappedPages object is "],["TableLocation","A record that tracks where an ACPI Table exists in memory, given in terms of offsets into the `AcpiTables`'s `MappedPages`."]],"type":[["AcpiSignature","All ACPI tables are identified by a 4-byte signature, typically an ASCII string like \"APIC\" or \"RSDT\"."]]});