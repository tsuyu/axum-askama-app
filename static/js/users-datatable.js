// Initialize DataTable for users list
$(document).ready(function () {
    const table = $('#usersTable').DataTable({
        // Enable server-side processing
        processing: true,
        serverSide: true,

        // AJAX configuration
        ajax: {
            url: '/api/v1/users/datatable',
            type: 'GET',
            error: function (xhr, error, thrown) {
                console.error('DataTables AJAX error:', error);
                alert('Failed to load user data. Please try again.');
            }
        },

        // Column definitions
        columns: [
            {
                data: 'id',
                title: 'ID',
                width: '10%'
            },
            {
                data: 'username',
                title: 'Username',
                width: '25%'
            },
            {
                data: 'email',
                title: 'Email',
                width: '30%'
            },
            {
                data: 'created_at',
                title: 'Created At',
                width: '25%',
                render: function (data, type, row) {
                    if (type === 'display' || type === 'filter') {
                        // Data is already formatted as 'dd-mm-yyyy hh:mm:ss' from backend
                        return data || 'N/A';
                    }
                    return data;
                }
            },
            {
                data: 'id',
                title: 'Actions',
                orderable: false,
                searchable: false,
                width: '10%',
                render: function (data, type, row) {
                    return (
                        '<a href="/admin/users/' + data + '" class="btn btn-sm btn-gradient me-1">View</a>' +
                        '<a href="/admin/users/' + data + '/edit" class="btn btn-sm btn-outline-secondary">Edit</a>'
                    );
                }
            }
        ],

        // Default ordering
        order: [[0, 'desc']],

        // Page length options
        lengthMenu: [[10, 25, 50, 100], [10, 25, 50, 100]],
        pageLength: 10,

        // Styling
        dom: '<"row"<"col-sm-12 col-md-6"l><"col-sm-12 col-md-6"f>>' +
            '<"row"<"col-sm-12"tr>>' +
            '<"row"<"col-sm-12 col-md-5"i><"col-sm-12 col-md-7"p>>',

        // Language customization
        language: {
            search: "_INPUT_",
            searchPlaceholder: "Search users...",
            lengthMenu: "Show _MENU_ entries",
            info: "Showing _START_ to _END_ of _TOTAL_ users",
            infoEmpty: "No users found",
            infoFiltered: "(filtered from _MAX_ total users)",
            zeroRecords: "No matching users found",
            emptyTable: "No users available",
            loadingRecords: "Loading...",
            processing: '<div class="spinner-border text-primary" role="status"><span class="visually-hidden">Loading...</span></div>'
        },

        // Responsive
        responsive: true
    });

    $('#exportPdfBtn').on('click', function () {
        const search = table.search();
        const order = table.order();
        const columnMap = ['id', 'username', 'email', 'created_at'];
        const orderColumn = columnMap[order[0][0]] || 'id';
        const orderDirection = order[0][1] || 'desc';

        const params = new URLSearchParams();
        if (search) {
            params.set('search', search);
        }
        params.set('order_column', orderColumn);
        params.set('order_direction', orderDirection);

        window.open('/admin/users/print?' + params.toString(), '_blank');
    });
});
