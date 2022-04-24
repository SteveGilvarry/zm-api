import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Users_Snapshots } from './users-snapshots.enum';
import { NestedEnumUsers_SnapshotsFilter } from './nested-enum-users-snapshots-filter.input';

@InputType()
export class EnumUsers_SnapshotsFilter {

    @Field(() => Users_Snapshots, {nullable:true})
    equals?: keyof typeof Users_Snapshots;

    @Field(() => [Users_Snapshots], {nullable:true})
    in?: Array<keyof typeof Users_Snapshots>;

    @Field(() => [Users_Snapshots], {nullable:true})
    notIn?: Array<keyof typeof Users_Snapshots>;

    @Field(() => NestedEnumUsers_SnapshotsFilter, {nullable:true})
    not?: NestedEnumUsers_SnapshotsFilter;
}
