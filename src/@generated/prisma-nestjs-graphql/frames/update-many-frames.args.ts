import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { FramesUpdateManyMutationInput } from './frames-update-many-mutation.input';
import { FramesWhereInput } from './frames-where.input';

@ArgsType()
export class UpdateManyFramesArgs {

    @Field(() => FramesUpdateManyMutationInput, {nullable:false})
    data!: FramesUpdateManyMutationInput;

    @Field(() => FramesWhereInput, {nullable:true})
    where?: FramesWhereInput;
}
