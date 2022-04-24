import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Snapshots } from './users-snapshots.enum';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumUsers_SnapshotsFilter } from './nested-enum-users-snapshots-filter.input';

@InputType()
export class NestedEnumUsers_SnapshotsWithAggregatesFilter {

    @Field(() => Users_Snapshots, {nullable:true})
    equals?: keyof typeof Users_Snapshots;

    @Field(() => [Users_Snapshots], {nullable:true})
    in?: Array<keyof typeof Users_Snapshots>;

    @Field(() => [Users_Snapshots], {nullable:true})
    notIn?: Array<keyof typeof Users_Snapshots>;

    @Field(() => NestedEnumUsers_SnapshotsWithAggregatesFilter, {nullable:true})
    not?: NestedEnumUsers_SnapshotsWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumUsers_SnapshotsFilter, {nullable:true})
    _min?: NestedEnumUsers_SnapshotsFilter;

    @Field(() => NestedEnumUsers_SnapshotsFilter, {nullable:true})
    _max?: NestedEnumUsers_SnapshotsFilter;
}
