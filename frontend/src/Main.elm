module Main exposing (main)

import Browser
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Html.Events exposing (on)
import Json.Decode as Decode
import Http
import Types exposing (..)
import Api


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , update = update
        , subscriptions = subscriptions
        , view = view
        }


init : () -> ( Model, Cmd Msg )
init _ =
    ( { page = Login
      , token = Nothing
      , userId = Nothing
      , posts = []
      , accounts = []
      , calendar = []
      , compose = emptyCompose
      , error = Nothing
      , success = Nothing
      , loginEmail = ""
      , loginPassword = ""
      , registerEmail = ""
      , registerPassword = ""
      , loading = False
      , publishing = Nothing
      , dashboardFilter = All
      , deleteConfirm = Nothing
      }
    , Cmd.none
    )


emptyCompose : ComposeModel
emptyCompose =
    { content = ""
    , selectedAccount = Nothing
    , scheduledAt = Nothing
    , platform = "threads"
    , mediaType = "TEXT"
    , mediaUrl = Nothing
    , aiPrompt = Nothing
    , aiGenerating = False
    }


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Navigate page ->
            let
                cmd =
                    case page of
                        Dashboard ->
                            Api.getPosts model.token GotPosts

                        Accounts ->
                            Api.getAccounts model.token GotAccounts

                        Calendar ->
                            Api.getCalendar model.token GotCalendar

                        _ ->
                            Cmd.none
            in
            ( { model | page = page, error = Nothing, success = Nothing }, cmd )

        LoginEmail email ->
            ( { model | loginEmail = email }, Cmd.none )

        LoginPassword password ->
            ( { model | loginPassword = password }, Cmd.none )

        DoLogin ->
            ( { model | loading = True }
            , Api.loginRequest model.loginEmail model.loginPassword LoginResult
            )

        LoginResult result ->
            case result of
                Ok token ->
                    ( { model
                        | token = Just token
                        , page = Dashboard
                        , loading = False
                        , loginEmail = ""
                        , loginPassword = ""
                      }
                    , Api.getPosts (Just token) GotPosts
                    )

                Err _ ->
                    ( { model
                        | error = Just "Invalid email or password"
                        , loading = False
                      }
                    , Cmd.none
                    )

        RegisterEmail email ->
            ( { model | registerEmail = email }, Cmd.none )

        RegisterPassword password ->
            ( { model | registerPassword = password }, Cmd.none )

        DoRegister ->
            ( { model | loading = True }
            , Api.registerRequest model.registerEmail model.registerPassword RegisterResult
            )

        RegisterResult result ->
            case result of
                Ok token ->
                    ( { model
                        | token = Just token
                        , page = Dashboard
                        , loading = False
                        , registerEmail = ""
                        , registerPassword = ""
                      }
                    , Api.getPosts (Just token) GotPosts
                    )

                Err _ ->
                    ( { model
                        | error = Just "Registration failed. Email may already be in use."
                        , loading = False
                      }
                    , Cmd.none
                    )

        Logout ->
            ( { model
                | token = Nothing
                , userId = Nothing
                , page = Login
                , posts = []
                , accounts = []
                , calendar = []
              }
            , Cmd.none
            )

        GotPosts result ->
            case result of
                Ok posts ->
                    ( { model | posts = posts }, Cmd.none )

                Err _ ->
                    ( { model | error = Just "Failed to load posts" }, Cmd.none )

        GotAccounts result ->
            case result of
                Ok accounts ->
                    ( { model | accounts = accounts }, Cmd.none )

                Err _ ->
                    ( { model | error = Just "Failed to load accounts" }, Cmd.none )

        GotCalendar result ->
            case result of
                Ok calendar ->
                    ( { model | calendar = calendar }, Cmd.none )

                Err _ ->
                    ( { model | error = Just "Failed to load calendar" }, Cmd.none )

        UpdateComposeContent content ->
            let
                compose =
                    model.compose
            in
            ( { model | compose = { compose | content = content } }, Cmd.none )

        UpdateComposePlatform platform ->
            let
                compose =
                    model.compose
            in
            ( { model | compose = { compose | platform = platform } }, Cmd.none )

        UpdateComposeAccount accountId ->
            let
                compose =
                    model.compose
            in
            ( { model | compose = { compose | selectedAccount = if String.isEmpty accountId then Nothing else Just accountId } }, Cmd.none )

        UpdateComposeMediaType mediaType ->
            let
                compose =
                    model.compose
            in
            ( { model | compose = { compose | mediaType = mediaType, mediaUrl = Nothing } }, Cmd.none )

        UpdateComposeMediaUrl url ->
            let
                compose =
                    model.compose
            in
            ( { model | compose = { compose | mediaUrl = if String.isEmpty url then Nothing else Just url } }, Cmd.none )

        UpdateComposeSchedule schedule ->
            let
                compose =
                    model.compose
            in
            ( { model | compose = { compose | scheduledAt = schedule } }, Cmd.none )

        UpdateAiPrompt prompt ->
            let
                compose =
                    model.compose
            in
            ( { model | compose = { compose | aiPrompt = Just prompt } }, Cmd.none )

        GenerateContent ->
            let
                compose =
                    model.compose

                cmd =
                    case compose.aiPrompt of
                        Just prompt ->
                            Api.generateContent model.token prompt compose.platform GotGeneratedContent

                        Nothing ->
                            Cmd.none
            in
            ( { model | compose = { compose | aiGenerating = True } }, cmd )

        GotGeneratedContent result ->
            let
                compose =
                    model.compose
            in
            case result of
                Ok content ->
                    ( { model | compose = { compose | content = content, aiGenerating = False } }, Cmd.none )

                Err _ ->
                    ( { model | compose = { compose | aiGenerating = False }, error = Just "Failed to generate content" }, Cmd.none )

        CreatePost ->
            let
                compose =
                    model.compose

                cmd =
                    case compose.selectedAccount of
                        Just accountId ->
                            Api.createPost model.token accountId compose.content compose.platform compose.scheduledAt PostCreated

                        Nothing ->
                            Cmd.none
            in
            ( model, cmd )

        PostCreated result ->
            case result of
                Ok _ ->
                    ( { model
                        | page = Dashboard
                        , compose = emptyCompose
                        , success = Just "Post created successfully!"
                      }
                    , Api.getPosts model.token GotPosts
                    )

                Err _ ->
                    ( { model | error = Just "Failed to create post" }, Cmd.none )

        PublishPost postId ->
            ( { model | publishing = Just postId }
            , Api.publishPost model.token postId PostPublished
            )

        PostPublished result ->
            let
                newModel =
                    { model | publishing = Nothing }
            in
            case result of
                Ok _ ->
                    ( { newModel | success = Just "Post published!" }
                    , Api.getPosts model.token GotPosts
                    )

                Err _ ->
                    ( { newModel | error = Just "Failed to publish post. Check your account connection." }, Cmd.none )

        DeletePost postId ->
            ( model, Api.deletePost model.token postId PostDeleted )

        PostDeleted result ->
            case result of
                Ok _ ->
                    ( { model | deleteConfirm = Nothing, success = Just "Post deleted." }
                    , Api.getPosts model.token GotPosts
                    )

                Err _ ->
                    ( { model | error = Just "Failed to delete post" }, Cmd.none )

        ConnectAccount ->
            ( model, Api.connectAccount model.token GotOAuthUrl )

        GotOAuthUrl result ->
            case result of
                Ok url ->
                    ( model, Http.get { url = url, expect = Http.expectWhatever (\_ -> DismissError) } )

                Err _ ->
                    ( { model | error = Just "Failed to start OAuth" }, Cmd.none )

        DeleteAccount accountId ->
            ( model, Api.deleteAccount model.token accountId AccountDeleted )

        AccountDeleted result ->
            case result of
                Ok _ ->
                    ( { model | success = Just "Account disconnected." }
                    , Api.getAccounts model.token GotAccounts
                    )

                Err _ ->
                    ( { model | error = Just "Failed to delete account" }, Cmd.none )

        SetDashboardFilter filter ->
            ( { model | dashboardFilter = filter }, Cmd.none )

        ShowDeleteConfirm postId ->
            ( { model | deleteConfirm = Just postId }, Cmd.none )

        HideDeleteConfirm ->
            ( { model | deleteConfirm = Nothing }, Cmd.none )

        ClearCompose ->
            ( { model | compose = emptyCompose }, Cmd.none )

        DismissError ->
            ( { model | error = Nothing }, Cmd.none )

        DismissSuccess ->
            ( { model | success = Nothing }, Cmd.none )


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.none


view : Model -> Html Msg
view model =
    div [ class "app" ]
        [ viewNavbar model
        , viewMessages model
        , viewPage model
        ]


viewNavbar : Model -> Html Msg
viewNavbar model =
    nav [ class "navbar" ]
        [ div [ class "navbar-brand" ] [ text "Poster" ]
        , if model.token /= Nothing then
            div [ class "navbar-menu" ]
                [ navLink Dashboard "Dashboard"
                , navLink Composer "Composer"
                , navLink Calendar "Calendar"
                , navLink Accounts "Accounts"
                , a [ class "nav-link logout", onClick Logout ] [ text "Logout" ]
                ]

          else
            text ""
        ]


navLink : Page -> String -> Html Msg
navLink page label =
    a [ class "nav-link", onClick (Navigate page) ] [ text label ]


viewMessages : Model -> Html Msg
viewMessages model =
    div [ class "messages" ]
        [ case model.error of
            Just msg ->
                div [ class "error-banner" ]
                    [ span [] [ text msg ]
                    , button [ onClick DismissError ] [ text "\u{00D7}" ]
                    ]

            Nothing ->
                text ""
        , case model.success of
            Just msg ->
                div [ class "success-banner" ]
                    [ span [] [ text msg ]
                    , button [ onClick DismissSuccess ] [ text "\u{00D7}" ]
                    ]

            Nothing ->
                text ""
        ]


viewPage : Model -> Html Msg
viewPage model =
    case model.page of
        Login ->
            viewLogin model

        Dashboard ->
            viewDashboard model

        Accounts ->
            viewAccounts model

        Composer ->
            viewComposer model

        Calendar ->
            viewCalendar model

        Analytics ->
            viewAnalytics

        Settings ->
            viewSettings


viewLogin : Model -> Html Msg
viewLogin model =
    div [ class "page login-page" ]
        [ div [ class "login-card" ]
            [ h1 [] [ text "Poster" ]
            , p [ class "subtitle" ] [ text "Marketing automation for Threads & Instagram" ]
            , div [ class "login-form" ]
                [ h2 [] [ text "Login" ]
                , input
                    [ placeholder "Email"
                    , type_ "email"
                    , value model.loginEmail
                    , onInput LoginEmail
                    ]
                    []
                , input
                    [ placeholder "Password"
                    , type_ "password"
                    , value model.loginPassword
                    , onInput LoginPassword
                    , onEnter DoLogin
                    ]
                    []
                , button
                    [ onClick DoLogin
                    , disabled model.loading
                    , class "btn-primary btn-full"
                    ]
                    [ text (if model.loading then "Logging in..." else "Login") ]
                , div [ class "divider" ] [ text "or" ]
                , h2 [] [ text "Register" ]
                , input
                    [ placeholder "Email"
                    , type_ "email"
                    , value model.registerEmail
                    , onInput RegisterEmail
                    ]
                    []
                , input
                    [ placeholder "Password (min 8 characters)"
                    , type_ "password"
                    , value model.registerPassword
                    , onInput RegisterPassword
                    , onEnter DoRegister
                    ]
                    []
                , button
                    [ onClick DoRegister
                    , disabled model.loading
                    , class "btn-secondary btn-full"
                    ]
                    [ text (if model.loading then "Registering..." else "Register") ]
                ]
            ]
        ]


onEnter : Msg -> Attribute Msg
onEnter msg =
    on "keydown"
        (Decode.field "key" Decode.string
            |> Decode.andThen
                (\key ->
                    if key == "Enter" then
                        Decode.succeed msg

                    else
                        Decode.fail "not enter"
                )
        )


viewDashboard : Model -> Html Msg
viewDashboard model =
    let
        filteredPosts =
            case model.dashboardFilter of
                All ->
                    model.posts

                Drafts ->
                    List.filter (\p -> p.status == "draft") model.posts

                Scheduled ->
                    List.filter (\p -> p.status == "scheduled") model.posts

                Published ->
                    List.filter (\p -> p.status == "published") model.posts

                Failed ->
                    List.filter (\p -> p.status == "failed") model.posts

        postCount status =
            List.length (List.filter (\p -> p.status == status) model.posts)
    in
    div [ class "page" ]
        [ div [ class "page-header" ]
            [ h1 [] [ text "Dashboard" ]
            , button [ onClick (Navigate Composer), class "btn-primary" ] [ text "+ New Post" ]
            ]
        , div [ class "filter-tabs" ]
            [ filterTab All "All" (List.length model.posts) model.dashboardFilter
            , filterTab Drafts "Drafts" (postCount "draft") model.dashboardFilter
            , filterTab Scheduled "Scheduled" (postCount "scheduled") model.dashboardFilter
            , filterTab Published "Published" (postCount "published") model.dashboardFilter
            , filterTab Failed "Failed" (postCount "failed") model.dashboardFilter
            ]
        , if List.isEmpty filteredPosts then
            div [ class "empty-state" ]
                [ text
                    (if List.isEmpty model.posts then
                        "No posts yet."

                     else
                        "No posts matching this filter."
                    )
                ]

          else
            div [ class "posts-list" ]
                (List.map (viewPostCard model) filteredPosts)
        ]


filterTab : PostFilter -> String -> Int -> PostFilter -> Html Msg
filterTab filter label count currentFilter =
    button
        [ class
            (if filter == currentFilter then
                "filter-tab active"

             else
                "filter-tab"
            )
        , onClick (SetDashboardFilter filter)
        ]
        [ text (label ++ " (" ++ String.fromInt count ++ ")") ]


viewPostCard : Model -> Post -> Html Msg
viewPostCard model post =
    let
        isPublishing =
            model.publishing == Just post.id

        isDeleting =
            model.deleteConfirm == Just post.id
    in
    div [ class ("post-card status-" ++ post.status) ]
        [ div [ class "post-header" ]
            [ div [ class "post-badges" ]
                [ span [ class ("status-badge status-" ++ post.status) ] [ text post.status ]
                , span [ class "platform-badge" ] [ text post.platform ]
                ]
            , div [ class "post-actions-top" ]
                [ if post.status == "draft" || post.status == "scheduled" then
                    button
                        [ onClick (PublishPost post.id)
                        , disabled isPublishing
                        , class "btn-primary btn-sm"
                        ]
                        [ text
                            (if isPublishing then
                                "Publishing..."

                             else
                                "Publish"
                            )
                        ]

                  else
                    text ""
                , button
                    [ onClick (ShowDeleteConfirm post.id)
                    , class "btn-ghost btn-sm"
                    ]
                    [ text "Delete" ]
                ]
            ]
        , div [ class "post-content" ] [ text post.content ]
        , case post.mediaUrl of
            Just url ->
                div [ class "post-media" ]
                    [ img [ src url, alt "Post media" ] []
                    ]

            Nothing ->
                text ""
        , div [ class "post-meta" ]
            [ case post.scheduledAt of
                Just scheduled ->
                    span [] [ text ("Scheduled: " ++ formatDateTime scheduled) ]

                Nothing ->
                    text ""
            , case post.publishedAt of
                Just published ->
                    span [] [ text ("Published: " ++ formatDateTime published) ]

                Nothing ->
                    text ""
            ]
        , if isDeleting then
            div [ class "delete-confirm" ]
                [ span [] [ text "Delete this post?" ]
                , button [ onClick (DeletePost post.id), class "btn-danger btn-sm" ] [ text "Yes, delete" ]
                , button [ onClick HideDeleteConfirm, class "btn-ghost btn-sm" ] [ text "Cancel" ]
                ]

          else
            text ""
        ]


formatDateTime : String -> String
formatDateTime dt =
    String.left 16 dt


viewAccounts : Model -> Html Msg
viewAccounts model =
    div [ class "page" ]
        [ div [ class "page-header" ]
            [ h1 [] [ text "Accounts" ]
            , button [ onClick ConnectAccount, class "btn-primary" ] [ text "+ Connect" ]
            ]
        , if List.isEmpty model.accounts then
            div [ class "empty-state" ]
                [ div [ class "empty-icon" ] [ text "\u{1F4F1}" ]
                , h3 [] [ text "No accounts connected" ]
                , p [] [ text "Connect your Instagram Business account to start posting." ]
                , button [ onClick ConnectAccount, class "btn-primary" ] [ text "Connect Instagram / Threads" ]
                ]

          else
            div [ class "accounts-grid" ]
                (List.map viewAccountCard model.accounts)
        ]


viewAccountCard : Account -> Html Msg
viewAccountCard account =
    div [ class "account-card" ]
        [ div [ class "account-icon" ]
            [ text
                (if account.provider == "instagram" then
                    "\u{1F4F7}"

                 else
                    "\u{1F4AC}"
                )
            ]
        , div [ class "account-info" ]
            [ div [ class "account-username" ] [ text account.username ]
            , div [ class "account-provider" ] [ text account.provider ]
            ]
        , div [ class "account-meta" ]
            [ case account.tokenExpiresAt of
                Just expires ->
                    div [ class "token-status" ]
                        [ span [ class "token-dot valid" ] []
                        , text ("Expires: " ++ formatDateTime expires)
                        ]

                Nothing ->
                    text ""
            ]
        , button
            [ class "btn-ghost btn-sm"
            , onClick (DeleteAccount account.id)
            ]
            [ text "Disconnect" ]
        ]


viewComposer : Model -> Html Msg
viewComposer model =
    let
        compose =
            model.compose

        canSubmit =
            compose.selectedAccount /= Nothing && not (String.isEmpty compose.content)
    in
    div [ class "page" ]
        [ div [ class "page-header" ]
            [ h1 [] [ text "Compose Post" ]
            , button [ onClick ClearCompose, class "btn-ghost" ] [ text "Clear" ]
            ]
        , div [ class "composer-layout" ]
            [ div [ class "composer-main" ]
                [ if List.isEmpty model.accounts then
                    div [ class "warning-banner" ]
                        [ text "Connect an account first in the "
                        , a [ onClick (Navigate Accounts) ] [ text "Accounts page" ]
                        , text "."
                        ]

                  else
                    div [ class "form-group" ]
                        [ label [] [ text "Account" ]
                        , select [ onInput UpdateComposeAccount ]
                            (option [ value "" ] [ text "Select account..." ]
                                :: List.map
                                    (\a ->
                                        option
                                            [ value a.id
                                            , selected (compose.selectedAccount == Just a.id)
                                            ]
                                            [ text (a.username ++ " (" ++ a.provider ++ ")") ]
                                    )
                                    model.accounts
                            )
                        ]
                , div [ class "form-row" ]
                    [ div [ class "form-group" ]
                        [ label [] [ text "Platform" ]
                        , select [ onInput UpdateComposePlatform ]
                            [ option [ value "threads", selected (compose.platform == "threads") ] [ text "Threads" ]
                            , option [ value "instagram", selected (compose.platform == "instagram") ] [ text "Instagram" ]
                            ]
                        ]
                    , div [ class "form-group" ]
                        [ label [] [ text "Media Type" ]
                        , select [ onInput UpdateComposeMediaType ]
                            [ option [ value "TEXT", selected (compose.mediaType == "TEXT") ] [ text "Text Only" ]
                            , option [ value "IMAGE", selected (compose.mediaType == "IMAGE") ] [ text "Image" ]
                            , option [ value "VIDEO", selected (compose.mediaType == "VIDEO") ] [ text "Video" ]
                            ]
                        ]
                    ]
                , div [ class "form-group" ]
                    [ label [] [ text "Content" ]
                    , textarea
                        [ placeholder "What's on your mind?"
                        , value compose.content
                        , onInput UpdateComposeContent
                        , class "composer-textarea"
                        ]
                        []
                    , div [ class "char-count" ]
                        [ text
                            (String.fromInt (String.length compose.content)
                                ++ "/500"
                            )
                        ]
                    ]
                , case compose.mediaType of
                    "IMAGE" ->
                        div [ class "form-group" ]
                            [ label [] [ text "Image URL" ]
                            , input
                                [ placeholder "https://example.com/image.jpg"
                                , onInput UpdateComposeMediaUrl
                                , value (compose.mediaUrl |> Maybe.withDefault "")
                                ]
                                []
                            ]

                    "VIDEO" ->
                        div [ class "form-group" ]
                            [ label [] [ text "Video URL" ]
                            , input
                                [ placeholder "https://example.com/video.mp4"
                                , onInput UpdateComposeMediaUrl
                                , value (compose.mediaUrl |> Maybe.withDefault "")
                                ]
                                []
                            ]

                    _ ->
                        text ""
                , div [ class "form-group" ]
                    [ label [] [ text "Schedule (optional)" ]
                    , input
                        [ type_ "datetime-local"
                        , onInput (\v -> UpdateComposeSchedule (if String.isEmpty v then Nothing else Just v))
                        ]
                        []
                    ]
                , div [ class "composer-actions" ]
                    [ button
                        [ onClick CreatePost
                        , disabled (not canSubmit)
                        , class "btn-primary"
                        ]
                        [ text "Save Draft" ]
                    ]
                ]
            , div [ class "composer-sidebar" ]
                [ div [ class "ai-section" ]
                    [ h3 [] [ text "AI Assistant" ]
                    , p [ class "ai-hint" ] [ text "Describe what you want to write about" ]
                    , textarea
                        [ placeholder "e.g. Write a post about our new product launch..."
                        , onInput UpdateAiPrompt
                        , value (compose.aiPrompt |> Maybe.withDefault "")
                        , class "ai-input"
                        ]
                        []
                    , button
                        [ onClick GenerateContent
                        , disabled compose.aiGenerating
                        , class "btn-secondary btn-full"
                        ]
                        [ text
                            (if compose.aiGenerating then
                                "Generating..."

                             else
                                "Generate with AI"
                            )
                        ]
                    ]
                , div [ class "preview-section" ]
                    [ h3 [] [ text "Preview" ]
                    , div [ class "post-preview" ]
                        [ div [ class "preview-content" ]
                            [ text
                                (if String.isEmpty compose.content then
                                    "Your post content will appear here..."

                                 else
                                    compose.content
                                )
                            ]
                        , div [ class "preview-meta" ]
                            [ span [] [ text compose.platform ]
                            ]
                        ]
                    ]
                ]
            ]
        ]


viewCalendar : Model -> Html Msg
viewCalendar model =
    div [ class "page" ]
        [ div [ class "page-header" ]
            [ h1 [] [ text "Content Calendar" ]
            , button [ onClick (Navigate Composer), class "btn-primary" ] [ text "+ New Post" ]
            ]
        , if List.isEmpty model.calendar then
            div [ class "empty-state" ]
                [ text "No scheduled posts." ]

          else
            div [ class "calendar-grid" ]
                (List.map viewCalendarDay model.calendar)
        ]


viewCalendarDay : CalendarDay -> Html Msg
viewCalendarDay day =
    div [ class "calendar-day" ]
        [ div [ class "calendar-date" ] [ text day.date ]
        , div [ class "calendar-posts" ]
            (List.map viewCalendarPost day.posts)
        ]


viewCalendarPost : Post -> Html Msg
viewCalendarPost post =
    div [ class ("calendar-post status-" ++ post.status) ]
        [ div [ class "calendar-post-time" ]
            [ text
                (case post.scheduledAt of
                    Just t ->
                        String.slice 11 16 t

                    Nothing ->
                        ""
                )
            ]
        , div [ class "calendar-post-content" ] [ text (String.left 50 post.content) ]
        , div [ class "calendar-post-meta" ]
            [ span [ class "platform-badge" ] [ text post.platform ]
            ]
        ]


viewAnalytics : Html Msg
viewAnalytics =
    div [ class "page" ]
        [ h1 [] [ text "Analytics" ]
        , div [ class "empty-state" ]
            [ text "Analytics coming soon. Connect an account to see insights." ]
        ]


viewSettings : Html Msg
viewSettings =
    div [ class "page" ]
        [ h1 [] [ text "Settings" ]
        , div [ class "empty-state" ]
            [ text "Settings coming soon." ]
        ]
