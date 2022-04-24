import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StorageWhereInput } from './storage-where.input';
import { StorageOrderByWithRelationInput } from './storage-order-by-with-relation.input';
import { StorageWhereUniqueInput } from './storage-where-unique.input';
import { Int } from '@nestjs/graphql';
import { StorageScalarFieldEnum } from './storage-scalar-field.enum';

@ArgsType()
export class FindManyStorageArgs {

    @Field(() => StorageWhereInput, {nullable:true})
    where?: StorageWhereInput;

    @Field(() => [StorageOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<StorageOrderByWithRelationInput>;

    @Field(() => StorageWhereUniqueInput, {nullable:true})
    cursor?: StorageWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [StorageScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof StorageScalarFieldEnum>;
}
