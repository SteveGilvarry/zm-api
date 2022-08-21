import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { SnapshotsWhereInput } from './snapshots-where.input';
import { Type } from 'class-transformer';
import { SnapshotsOrderByWithRelationInput } from './snapshots-order-by-with-relation.input';
import { SnapshotsWhereUniqueInput } from './snapshots-where-unique.input';
import { Int } from '@nestjs/graphql';
import { SnapshotsScalarFieldEnum } from './snapshots-scalar-field.enum';

@ArgsType()
export class FindFirstSnapshotsArgs {

    @Field(() => SnapshotsWhereInput, {nullable:true})
    @Type(() => SnapshotsWhereInput)
    where?: SnapshotsWhereInput;

    @Field(() => [SnapshotsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<SnapshotsOrderByWithRelationInput>;

    @Field(() => SnapshotsWhereUniqueInput, {nullable:true})
    cursor?: SnapshotsWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [SnapshotsScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof SnapshotsScalarFieldEnum>;
}
