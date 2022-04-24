import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Monitors_DefaultCodec } from '../monitors/monitors-default-codec.enum';
import { NestedEnumMonitors_DefaultCodecWithAggregatesFilter } from './nested-enum-monitors-default-codec-with-aggregates-filter.input';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumMonitors_DefaultCodecFilter } from './nested-enum-monitors-default-codec-filter.input';

@InputType()
export class EnumMonitors_DefaultCodecWithAggregatesFilter {

    @Field(() => Monitors_DefaultCodec, {nullable:true})
    equals?: keyof typeof Monitors_DefaultCodec;

    @Field(() => [Monitors_DefaultCodec], {nullable:true})
    in?: Array<keyof typeof Monitors_DefaultCodec>;

    @Field(() => [Monitors_DefaultCodec], {nullable:true})
    notIn?: Array<keyof typeof Monitors_DefaultCodec>;

    @Field(() => NestedEnumMonitors_DefaultCodecWithAggregatesFilter, {nullable:true})
    not?: NestedEnumMonitors_DefaultCodecWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumMonitors_DefaultCodecFilter, {nullable:true})
    _min?: NestedEnumMonitors_DefaultCodecFilter;

    @Field(() => NestedEnumMonitors_DefaultCodecFilter, {nullable:true})
    _max?: NestedEnumMonitors_DefaultCodecFilter;
}
