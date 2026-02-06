$(document).ready(function () {
    const countrySelect = $('#country_id');
    const stateSelect = $('#state_id');

    async function loadStates(countryId) {
        stateSelect.prop('disabled', true);
        stateSelect.empty();
        stateSelect.append('<option value="">Select state</option>');

        if (!countryId) {
            stateSelect.prop('disabled', false);
            return;
        }

        try {
            const resp = await fetch('/admin/geo/states?country_id=' + encodeURIComponent(countryId));
            if (!resp.ok) {
                throw new Error('Failed to load states');
            }
            const states = await resp.json();
            const selected = stateSelect.data('selected');

            states.forEach((s) => {
                const option = $('<option></option>')
                    .attr('value', s.id)
                    .text(s.name);
                if (selected && Number(selected) === s.id) {
                    option.attr('selected', 'selected');
                }
                stateSelect.append(option);
            });
        } catch (e) {
            console.error(e);
        } finally {
            stateSelect.prop('disabled', false);
        }
    }

    countrySelect.on('change', function () {
        const countryId = $(this).val();
        stateSelect.data('selected', null);
        loadStates(countryId);
    });

    // initial load
    loadStates(countrySelect.val());
});
