module Main exposing (main)

import Browser
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
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
      , compose =
            { content = ""
            , selectedAccount = Nothing
            , scheduledAt = Nothing
            , platform = "threads"
            , mediaType = "TEXT"
            , mediaUrl = Nothing
            , aiPrompt = Nothing
            , aiGenerating = False
            }
      , error = Nothing
      , loginEmail = ""
      , loginPassword = ""
      , registerEmail = ""
      , registerPassword = ""
      , loading = False
      , publishing = Nothing
      }
    , Cmd.none
    )


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
            ( { model | page = page }, cmd )

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
            ( { model | compose = { compose | selectedAccount = Just accountId } }, Cmd.none )

        UpdateComposeMediaType mediaType ->
            let
                compose =
                    model.compose
            in
            ( { model | compose = { compose | mediaType = mediaType } }, Cmd.none )

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
                            Api.createPost model.token accountId compose.content compose.scheduledAt compose.platform PostCreated

                        Nothing ->
                            Cmd.none
            in
            ( model, cmd )

        PostCreated result ->
            case result of
                Ok _ ->
                    ( { model
                        | page = Dashboard
                        , compose =
                            { content = ""
                            , selectedAccount = Nothing
                            , scheduledAt = Nothing
                            , platform = "threads"
                            , mediaType = "TEXT"
                            , mediaUrl = Nothing
                            , aiPrompt = Nothing
                            , aiGenerating = False
                            }
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
                    ( newModel, Api.getPosts model.token GotPosts )

                Err _ ->
                    ( { newModel | error = Just "Failed to publish post. Check your account connection." }, Cmd.none )

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
                    ( model, Api.getAccounts model.token GotAccounts )

                Err _ ->
                    ( { model | error = Just "Failed to delete account" }, Cmd.none )

        ClearCompose ->
            ( { model
                | compose =
                    { content = ""
                    , selectedAccount = Nothing
                    , scheduledAt = Nothing
                    , platform = "threads"
                    , mediaType = "TEXT"
                    , mediaUrl = Nothing
                    , aiPrompt = Nothing
                    , aiGenerating = False
                    }
              }
            , Cmd.none
            )

        DismissError ->
            ( { model | error = Nothing }, Cmd.none )


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.none


view : Model -> Html Msg
view model =
    div [ class "app" ]
        [ viewNavbar model
        , viewError model.error
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
                , navLink Analytics "Analytics"
                , a [ class "nav-link", onClick Logout ] [ text "Logout" ]
                ]

          else
            text ""
        ]


navLink : Page -> String -> Html Msg
navLink page label =
    a [ class "nav-link", onClick (Navigate page) ] [ text label ]


viewError : Maybe String -> Html Msg
viewError error =
    case error of
        Just msg ->
            div [ class "error-banner" ]
                [ text msg
                , button [ onClick DismissError ] [ text "x" ]
                ]

        Nothing ->
            text ""


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
                ]
                []
            , button
                [ onClick DoLogin
                , disabled model.loading
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
                ]
                []
            , button
                [ onClick DoRegister
                , disabled model.loading
                ]
                [ text (if model.loading then "Registering..." else "Register") ]
            ]
        ]


viewDashboard : Model -> Html Msg
viewDashboard model =
    div [ class "page" ]
        [ div [ class "page-header" ]
            [ h1 [] [ text "Dashboard" ]
            , button [ onClick (Navigate Composer), class "btn-primary" ] [ text "+ New Post" ]
            ]
        , if List.isEmpty model.posts then
            div [ class "empty-state" ]
                [ p [] [ text "No posts yet." ]
                , button [ onClick (Navigate Composer), class "btn-primary" ] [ text "Create your first post" ]
                ]

          else
            div [ class "posts-list" ]
                (List.map (viewPostCard model.publishing) model.posts)
        ]


viewPostCard : Maybe String -> Post -> Html Msg
viewPostCard publishing post =
    let
        isPublishing =
            publishing == Just post.id
    in
    div [ class ("post-card " ++ post.status) ]
        [ div [ class "post-header" ]
            [ span [ class ("status-badge status-" ++ post.status) ] [ text post.status ]
            , span [ class "platform-badge" ] [ text post.platform ]
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
                    div [] [ text ("Scheduled: " ++ scheduled) ]

                Nothing ->
                    text ""
            , case post.publishedAt of
                Just published ->
                    div [] [ text ("Published: " ++ published) ]

                Nothing ->
                    text ""
            ]
        , div [ class "post-actions" ]
            [ if post.status == "draft" || post.status == "scheduled" then
                button
                    [ onClick (PublishPost post.id)
                    , disabled isPublishing
                    , class "btn-primary"
                    ]
                    [ text
                        (if isPublishing then
                            "Publishing..."

                         else
                            "Publish Now"
                        )
                    ]

              else
                text ""
            ]
        ]


viewAccounts : Model -> Html Msg
viewAccounts model =
    div [ class "page" ]
        [ h1 [] [ text "Accounts" ]
        , button [ onClick ConnectAccount, class "btn-primary" ] [ text "+ Connect Instagram / Threads" ]
        , if List.isEmpty model.accounts then
            div [ class "empty-state" ]
                [ p [] [ text "No accounts connected." ]
                , p [] [ text "Connect your Instagram Business account to start posting." ]
                ]

          else
            div [ class "accounts-list" ]
                (List.map viewAccountCard model.accounts)
        ]


viewAccountCard : Account -> Html Msg
viewAccountCard account =
    div [ class "account-card" ]
        [ div [ class "account-header" ]
            [ div [ class "account-info" ]
                [ div [ class "account-username" ] [ text account.username ]
                , div [ class "account-provider" ] [ text account.provider ]
                ]
            , button
                [ class "btn-danger"
                , onClick (DeleteAccount account.id)
                ]
                [ text "Disconnect" ]
            ]
        , div [ class "account-details" ]
            [ div [] [ text ("User ID: " ++ account.providerUserId) ]
            , case account.tokenExpiresAt of
                Just expires ->
                    div [] [ text ("Token expires: " ++ expires) ]

                Nothing ->
                    text ""
            ]
        ]


viewComposer : Model -> Html Msg
viewComposer model =
    let
        compose =
            model.compose
    in
    div [ class "page" ]
        [ div [ class "page-header" ]
            [ h1 [] [ text "Compose Post" ]
            , button [ onClick ClearCompose, class "btn-secondary" ] [ text "Clear" ]
            ]
        , div [ class "composer" ]
            [ if List.isEmpty model.accounts then
                div [ class "warning" ]
                    [ text "Connect an account first in the "
                    , a [ onClick (Navigate Accounts) ] [ text "Accounts" ]
                    , text " page."
                    ]

              else
                div [ class "form-group" ]
                    [ label [] [ text "Account" ]
                    , select [ onInput UpdateComposeAccount ]
                        (option [ value "" ] [ text "Select account..." ]
                            :: List.map
                                (\a ->
                                    option [ value a.id ]
                                        [ text (a.username ++ " (" ++ a.provider ++ ")") ]
                                )
                                model.accounts
                        )
                    ]
            , div [ class "form-group" ]
                [ label [] [ text "Platform" ]
                , select [ onInput UpdateComposePlatform ]
                    [ option [ value "threads", selected (compose.platform == "threads") ] [ text "Threads" ]
                    , option [ value "instagram", selected (compose.platform == "instagram") ] [ text "Instagram" ]
                    ]
                ]
            , div [ class "form-group" ]
                [ label [] [ text "Content" ]
                , textarea
                    [ placeholder "What's on your mind?"
                    , value compose.content
                    , onInput UpdateComposeContent
                    ]
                    []
                , div [ class "char-count" ]
                    [ text (String.fromInt (String.length compose.content) ++ "/500 characters") ]
                ]
            , div [ class "form-group" ]
                [ label [] [ text "Media Type" ]
                , select [ onInput UpdateComposeMediaType ]
                    [ option [ value "TEXT", selected (compose.mediaType == "TEXT") ] [ text "Text Only" ]
                    , option [ value "IMAGE", selected (compose.mediaType == "IMAGE") ] [ text "Image" ]
                    , option [ value "VIDEO", selected (compose.mediaType == "VIDEO") ] [ text "Video" ]
                    ]
                ]
            , case compose.mediaType of
                "IMAGE" ->
                    div [ class "form-group" ]
                        [ label [] [ text "Image URL" ]
                        , input
                            [ placeholder "https://example.com/image.jpg"
                            , onInput UpdateComposeMediaUrl
                            ]
                            []
                        ]

                "VIDEO" ->
                    div [ class "form-group" ]
                        [ label [] [ text "Video URL" ]
                        , input
                            [ placeholder "https://example.com/video.mp4"
                            , onInput UpdateComposeMediaUrl
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
                    , disabled (compose.selectedAccount == Nothing || String.isEmpty compose.content)
                    , class "btn-primary"
                    ]
                    [ text "Save Draft" ]
                ]
            , div [ class "ai-section" ]
                [ h3 [] [ text "AI Assistant" ]
                , input
                    [ placeholder "Describe what to write about..."
                    , onInput UpdateAiPrompt
                    ]
                    []
                , button
                    [ onClick GenerateContent
                    , disabled compose.aiGenerating
                    , class "btn-secondary"
                    ]
                    [ text (if compose.aiGenerating then "Generating..." else "Generate with AI") ]
                ]
            ]
        ]


viewCalendar : Model -> Html Msg
viewCalendar model =
    div [ class "page" ]
        [ h1 [] [ text "Content Calendar" ]
        , if List.isEmpty model.calendar then
            p [] [ text "No scheduled posts." ]

          else
            div [ class "calendar" ]
                (List.map viewCalendarDay model.calendar)
        ]


viewCalendarDay : CalendarDay -> Html Msg
viewCalendarDay day =
    div [ class "calendar-day" ]
        [ h3 [] [ text day.date ]
        , div [ class "calendar-posts" ]
            (List.map (viewPostCard Nothing) day.posts)
        ]


viewAnalytics : Html Msg
viewAnalytics =
    div [ class "page" ]
        [ h1 [] [ text "Analytics" ]
        , p [] [ text "Connect an account to see analytics." ]
        ]


viewSettings : Html Msg
viewSettings =
    div [ class "page" ]
        [ h1 [] [ text "Settings" ]
        , p [] [ text "Settings coming soon." ]
        ]
