$(document).ready(function () {
    const countrySelect = $('#country_id');
    const stateSelect = $('#state_id');
    const selectedStateId = stateSelect.data('selected'); // Store the original selected state

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

            states.forEach((s) => {
                const option = $('<option></option>')
                    .attr('value', s.id)
                    .text(s.name);
                if (selectedStateId && Number(selectedStateId) === s.id) {
                    option.attr('selected', 'selected');
                }
                stateSelect.append(option);
            });
        } catch (e) {
            console.error('Failed to load states:', e);
            alert('Failed to load states. Please try again.');
        } finally {
            stateSelect.prop('disabled', false);
        }
    }

    countrySelect.on('change', function () {
        const countryId = $(this).val();
        loadStates(countryId);
    });

    // Only load states on page load if no states are pre-rendered or if country is empty
    const hasStates = stateSelect.find('option').length > 1;
    if (!hasStates && countrySelect.val()) {
        loadStates(countrySelect.val());
    }
});

