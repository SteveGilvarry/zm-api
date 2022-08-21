import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FramesUpdateManyMutationInput } from './frames-update-many-mutation.input';
import { Type } from 'class-transformer';
import { FramesWhereInput } from './frames-where.input';

@ArgsType()
export class UpdateManyFramesArgs {

    @Field(() => FramesUpdateManyMutationInput, {nullable:false})
    @Type(() => FramesUpdateManyMutationInput)
    data!: FramesUpdateManyMutationInput;

    @Field(() => FramesWhereInput, {nullable:true})
    @Type(() => FramesWhereInput)
    where?: FramesWhereInput;
}
