import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FramesWhereInput } from './frames-where.input';
import { FramesOrderByWithRelationInput } from './frames-order-by-with-relation.input';
import { FramesWhereUniqueInput } from './frames-where-unique.input';
import { Int } from '@nestjs/graphql';
import { FramesScalarFieldEnum } from './frames-scalar-field.enum';

@ArgsType()
export class FindManyFramesArgs {

    @Field(() => FramesWhereInput, {nullable:true})
    where?: FramesWhereInput;

    @Field(() => [FramesOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<FramesOrderByWithRelationInput>;

    @Field(() => FramesWhereUniqueInput, {nullable:true})
    cursor?: FramesWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [FramesScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof FramesScalarFieldEnum>;
}
