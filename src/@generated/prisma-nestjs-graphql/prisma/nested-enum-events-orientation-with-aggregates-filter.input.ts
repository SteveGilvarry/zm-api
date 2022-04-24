import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Events_Orientation } from '../events/events-orientation.enum';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumEvents_OrientationFilter } from './nested-enum-events-orientation-filter.input';

@InputType()
export class NestedEnumEvents_OrientationWithAggregatesFilter {

    @Field(() => Events_Orientation, {nullable:true})
    equals?: keyof typeof Events_Orientation;

    @Field(() => [Events_Orientation], {nullable:true})
    in?: Array<keyof typeof Events_Orientation>;

    @Field(() => [Events_Orientation], {nullable:true})
    notIn?: Array<keyof typeof Events_Orientation>;

    @Field(() => NestedEnumEvents_OrientationWithAggregatesFilter, {nullable:true})
    not?: NestedEnumEvents_OrientationWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumEvents_OrientationFilter, {nullable:true})
    _min?: NestedEnumEvents_OrientationFilter;

    @Field(() => NestedEnumEvents_OrientationFilter, {nullable:true})
    _max?: NestedEnumEvents_OrientationFilter;
}
